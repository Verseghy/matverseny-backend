package handler

import (
	"context"
	"time"

	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/bson/primitive"
	"go.mongodb.org/mongo-driver/mongo"
	"go.mongodb.org/mongo-driver/mongo/options"
	"go.mongodb.org/mongo-driver/x/bsonx"
	"go.uber.org/zap"
	"matverseny-backend/entity"
	"matverseny-backend/errs"
	"matverseny-backend/events"
	"matverseny-backend/jwt"
	"matverseny-backend/log"
	pb "matverseny-backend/proto"
)

const (
	timePeriod = 30 * time.Second
)

type superAdminHandler struct {
	cInfo     *mongo.Collection
	cHistory  *mongo.Collection
	cProblems *mongo.Collection
	jwt       jwt.JWT

	pb.UnimplementedSuperAdminServer
}

func (h *superAdminHandler) AuthFuncOverride(ctx context.Context, fullMethodName string) (context.Context, error) {
	f := h.jwt.ValidateSuperAdminToken()
	ctx, err := f(ctx)
	if err != nil {
		return nil, err
	}

	return ctx, nil
}

func (h *superAdminHandler) SetTime(ctx context.Context, req *pb.SetTimeRequest) (*pb.SetTimeResponse, error) {
	res := &pb.SetTimeResponse{}

	start, err := time.Parse(time.RFC3339, req.Start)
	if err != nil {
		return nil, errs.ErrInvalidTime
	}

	end, err := time.Parse(time.RFC3339, req.End)
	if err != nil {
		return nil, errs.ErrInvalidTime
	}

	t := &entity.Time{
		StartDate: start,
		EndDate:   end,
	}
	_, err = h.cInfo.UpdateOne(ctx, bson.M{}, bson.M{"$set": bson.M{"time": t}}, options.Update().SetUpsert(true))
	if err != nil {
		log.Logger.Error("database error", zap.Error(err))
		return nil, errs.ErrDatabase
	}

	err = events.PublishTime(&events.TimeEvent{
		Start: &start,
		End:   &end,
	})
	if err != nil {
		return nil, errs.ErrQueue
	}

	return res, nil
}

func (h *superAdminHandler) GetTime(ctx context.Context, req *pb.GetTimeRequest) (*pb.GetTimeResponse, error) {
	t := &entity.Info{}
	err := h.cInfo.FindOne(ctx, bson.M{}).Decode(t)
	if err != nil {
		log.Logger.Error("database error", zap.Error(err))
		return nil, errs.ErrDatabase
	}

	res := &pb.GetTimeResponse{
		Start: t.Time.StartDate.Format(time.RFC3339),
		End:   t.Time.EndDate.Format(time.RFC3339),
	}

	return res, nil
}

func calculatePoints(correctSolutions map[primitive.ObjectID]int64, currentSolutions map[primitive.ObjectID]map[primitive.ObjectID]int64) map[primitive.ObjectID]uint32 {
	logger := log.Logger
	points := make(map[primitive.ObjectID]uint32)

	for team, solutions := range currentSolutions {
		for problem, solution := range solutions {
			if correctSolution, ok := correctSolutions[problem]; ok {
				if correctSolution == solution {
					points[team]++
				}
			} else {
				logger.Warn("no correct solution found for problemID", zap.String("problemID", problem.Hex()))
			}
		}
	}

	return points
}

func (h *superAdminHandler) GetResults(req *pb.GetResultsRequest, stream pb.SuperAdmin_GetResultsServer) error {
	ctx, cancel := context.WithCancel(stream.Context())
	defer cancel()
	logger := log.Logger

	chSolution, err := events.ConsumeAdminSolution(ctx)
	if err != nil {
		logger.Error("queue error", zap.Error(err))
		return errs.ErrQueue
	}

	chProblems, err := events.ConsumeProblem(ctx)
	if err != nil {
		logger.Error("queue error", zap.Error(err))
		return errs.ErrQueue
	}

	go func() {
	L1:
		for {
			select {
			case <-ctx.Done():
				break L1

			// If the problem changes, we need to recalculate everything. Better to close the connection and restart this process.
			case <-chProblems:
				cancel()
			}
		}
	}()

	t := &entity.Info{}
	err = h.cInfo.FindOne(ctx, bson.M{}).Decode(t)
	if err != nil {
		logger.Error("database error", zap.Error(err))
		return errs.ErrDatabase
	}

	problems := make([]*entity.Problem, 0)
	find, err := h.cProblems.Find(ctx, bson.M{})
	if err != nil {
		logger.Error("database error", zap.Error(err))
		return errs.ErrDatabase
	}
	err = find.All(ctx, &problems)
	if err != nil {
		logger.Error("database error", zap.Error(err))
		return errs.ErrDatabase
	}
	correctSolutions := make(map[primitive.ObjectID]int64)
	for _, problem := range problems {
		correctSolutions[problem.ID] = problem.Solution
	}
	//                               teamID                 problemID
	currentSolution := make(map[primitive.ObjectID]map[primitive.ObjectID]int64)
	currentTimeBucket := t.Time.StartDate

	sendResponse := func() error {
		points := calculatePoints(correctSolutions, currentSolution)
		err = stream.Send(&pb.GetResultsResponse{
			Timestamp: uint32(currentTimeBucket.Unix()),
			Results: func() map[string]*pb.GetResultsResponse_Result {
				res := make(map[string]*pb.GetResultsResponse_Result)
				for team, point := range points {
					res[team.Hex()] = &pb.GetResultsResponse_Result{
						TotalAnswered:        uint32(len(currentSolution[team])),
						SuccessfullyAnswered: point,
					}
				}
				return res
			}(),
		})
		if err != nil {
			logger.Debug("sending failed", zap.Error(err))
			return err
		}

		currentTimeBucket = currentTimeBucket.Add(timePeriod)
		return nil
	}

	cursor, err := h.cHistory.Find(ctx, bson.M{}, options.Find().SetSort(&bson.M{"time": 1}))
	if err != nil {
		logger.Error("database error", zap.Error(err))
		return errs.ErrDatabase
	}
	defer cursor.Close(context.Background())

	for cursor.Next(ctx) {
		s := &entity.History{}
		err := cursor.Decode(s)
		if err != nil {
			logger.Error("decode error", zap.Error(err))
			return errs.ErrDatabase
		}

		for s.Time.After(currentTimeBucket) {
			err := sendResponse()
			if err != nil {
				return err
			}
		}

		if _, ok := currentSolution[s.Team]; !ok {
			currentSolution[s.Team] = make(map[primitive.ObjectID]int64)
		}
		currentSolution[s.Team][s.ProblemID] = s.Value
	}
	if err := cursor.Err(); err != nil {
		logger.Error("cursor error", zap.Error(err))
		return errs.ErrDatabase
	}

L1:
	for {
		select {
		case <-ctx.Done():
			break L1
		case s := <-chSolution:
			for time.Now().After(currentTimeBucket) {
				err := sendResponse()
				if err != nil {
					return err
				}
			}

			if _, ok := currentSolution[s.Team]; !ok {
				currentSolution[s.Team] = make(map[primitive.ObjectID]int64)
			}
			currentSolution[s.Team][s.ProblemID] = s.Value
		}
	}

	return nil
}

func NewSuperAdminHandler(client *mongo.Client) *superAdminHandler {
	_, err := client.Database("comp").Collection("history").Indexes().CreateMany(context.Background(), []mongo.IndexModel{
		{Keys: bsonx.Doc{{Key: "time", Value: bsonx.Int32(1)}}},
	})
	if err != nil {
		log.Logger.Fatal("unable to create index", zap.Error(err))
	}

	return &superAdminHandler{
		cInfo:     client.Database("comp").Collection("info"),
		cHistory:  client.Database("comp").Collection("history"),
		cProblems: client.Database("comp").Collection("problems"),
		jwt:       jwt.NewJWT(client, []byte("test-key")),
	}
}
