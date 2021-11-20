package handler

import (
	"context"
	"time"

	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/bson/primitive"
	"go.mongodb.org/mongo-driver/mongo"
	"go.mongodb.org/mongo-driver/mongo/options"
	"go.uber.org/zap"
	"matverseny-backend/entity"
	"matverseny-backend/errs"
	"matverseny-backend/events"
	"matverseny-backend/jwt"
	"matverseny-backend/log"
	pb "matverseny-backend/proto"
)

type competitionHandler struct {
	cSolutions *mongo.Collection
	cProblems  *mongo.Collection
	cInfo      *mongo.Collection
	cHistory   *mongo.Collection
	jwt        jwt.JWT

	pb.UnimplementedCompetitionServer
}

func (h *competitionHandler) AuthFuncOverride(ctx context.Context, fullMethodName string) (context.Context, error) {
	allowedWithoutAuthentication := []string{
		"/competition.Competition/GetTimes",
	}

	for _, v := range allowedWithoutAuthentication {
		if v == fullMethodName {
			return ctx, nil
		}
	}

	f := h.jwt.ValidateAccessToken()
	ctx, err := f(ctx)
	if err != nil {
		return nil, err
	}

	claims, ok := jwt.GetClaimsFromCtx(ctx)
	if !ok {
		return nil, errs.ErrJWT
	}

	if claims.Team == "" {
		return nil, errs.ErrNoTeam
	}
	logger := log.Logger.With(zap.String("userID", claims.UserID))

	t := &entity.Info{}
	err = h.cInfo.FindOne(ctx, bson.M{}).Decode(t)
	if err != nil {
		logger.Error("database error", zap.Error(err))
		return nil, errs.ErrDatabase
	}

	for t.Time.StartDate.After(time.Now()) {
		time.Sleep(t.Time.StartDate.Sub(time.Now()))
	}

	return ctx, nil
}

func (h *competitionHandler) GetProblems(req *pb.ProblemStreamRequest, stream pb.Competition_GetProblemsServer) error {
	claims, ok := jwt.GetClaimsFromCtx(stream.Context())
	if !ok {
		log.Logger.Error("jwt had no data")
		return errs.ErrJWT
	}
	logger := log.Logger.With(zap.String("userID", claims.UserID))

	ch, err := events.ConsumeProblem(stream.Context())
	if err != nil {
		logger.Error("queue error", zap.Error(err))
		return errs.ErrQueue
	}

	cursor, err := h.cProblems.Find(stream.Context(), bson.M{})
	if err != nil {
		logger.Error("database error", zap.Error(err))
		return errs.ErrDatabase
	}
	defer cursor.Close(context.Background())

	for cursor.Next(stream.Context()) {
		p := &entity.Problem{}
		err := cursor.Decode(p)
		if err != nil {
			logger.Error("decode error", zap.Error(err))
			return errs.ErrDatabase
		}

		err = stream.Send(&pb.ProblemStream{
			Type: pb.ProblemStream_k_INITIAL,
			Initial: &pb.ProblemStream_Initial{
				Problem: &pb.Problem{
					Id:    p.ID.Hex(),
					Body:  p.Body,
					Image: p.Image,
				},
				At: p.Position,
			},
		})
		if err != nil {
			logger.Debug("sending failed", zap.Error(err))
			return err
		}
	}
	if err := cursor.Err(); err != nil {
		logger.Error("cursor error", zap.Error(err))
		return errs.ErrDatabase
	}

L1:
	for {
		select {
		case <-stream.Context().Done():
			break L1
		case p := <-ch:
			err = stream.Send(&pb.ProblemStream{
				Type: func() pb.ProblemStream_Type {
					if p.Type == events.PChange {
						return pb.ProblemStream_k_UPDATE
					}
					if p.Type == events.PDelete {
						return pb.ProblemStream_k_DELETE
					}
					if p.Type == events.PSwap {
						return pb.ProblemStream_k_SWAP
					}
					if p.Type == events.PCreate {
						return pb.ProblemStream_k_CREATE
					}

					panic("encountered unknown problem type")
				}(),
				Initial: nil,
				Update: func() *pb.ProblemStream_Update {
					if p.Type != events.PChange {
						return nil
					}

					return &pb.ProblemStream_Update{Problem: p.Problem.ToProto()}
				}(),
				Delete: func() *pb.ProblemStream_Delete {
					if p.Type != events.PDelete {
						return nil
					}

					return &pb.ProblemStream_Delete{Id: p.Problem.ID.Hex()}
				}(),
				Swap: func() *pb.ProblemStream_Swap {
					if p.Type != events.PSwap {
						return nil
					}

					return &pb.ProblemStream_Swap{A: p.A.ID.Hex(), B: p.B.ID.Hex()}
				}(),
				Create: func() *pb.ProblemStream_Create {
					if p.Type != events.PCreate {
						return nil
					}

					return &pb.ProblemStream_Create{At: p.At, Problem: p.Problem.ToProto()}
				}(),
			})
			if err != nil {
				logger.Debug("sending failed", zap.Error(err))
				return err
			}
		}
	}

	return nil
}
func (h *competitionHandler) GetSolutions(req *pb.GetSolutionsRequest, stream pb.Competition_GetSolutionsServer) error {
	claims, ok := jwt.GetClaimsFromCtx(stream.Context())
	if !ok {
		log.Logger.Error("jwt had no data")
		return errs.ErrJWT
	}
	logger := log.Logger.With(zap.String("userID", claims.UserID))

	teamID, err := primitive.ObjectIDFromHex(claims.Team)
	if err != nil {
		log.Logger.Error("invalid team id", zap.Error(err))
		return errs.ErrJWT
	}

	ch, err := events.ConsumeSolution(stream.Context(), teamID.Hex())
	if err != nil {
		logger.Error("queue error", zap.Error(err))
		return errs.ErrQueue
	}

	cursor, err := h.cSolutions.Find(stream.Context(), bson.M{"team": teamID})
	if err != nil {
		logger.Error("database error", zap.Error(err))
		return errs.ErrDatabase
	}
	defer cursor.Close(context.Background())

	for cursor.Next(stream.Context()) {
		s := &entity.Solution{}
		err := cursor.Decode(s)
		if err != nil {
			logger.Error("decode error", zap.Error(err))
			return errs.ErrDatabase
		}

		err = stream.Send(&pb.GetSolutionsResponse{
			Id:    s.ProblemID.Hex(),
			Type:  pb.GetSolutionsResponse_k_CHANGE,
			Value: s.Value,
		})
		if err != nil {
			logger.Debug("sending failed", zap.Error(err))
			return err
		}
	}
	if err := cursor.Err(); err != nil {
		logger.Error("cursor error", zap.Error(err))
		return errs.ErrDatabase
	}

L1:
	for {
		select {
		case <-stream.Context().Done():
			break L1
		case s := <-ch:
			err = stream.Send(&pb.GetSolutionsResponse{
				Id: s.ProblemID.Hex(),
				Type: func() pb.GetSolutionsResponse_Modification {
					if s.Type == events.SChange {
						return pb.GetSolutionsResponse_k_CHANGE
					}
					if s.Type == events.SDelete {
						return pb.GetSolutionsResponse_k_DELETE
					}

					panic("encountered invalid modification type")
				}(),
				Value: s.Value,
			})
			if err != nil {
				logger.Debug("sending failed", zap.Error(err))
				return err
			}
		}
	}

	return nil
}
func (h *competitionHandler) SetSolutions(ctx context.Context, req *pb.SetSolutionsRequest) (*pb.SetSolutionsResponse, error) {
	res := &pb.SetSolutionsResponse{}

	claims, ok := jwt.GetClaimsFromCtx(ctx)
	if !ok {
		log.Logger.Error("jwt had no data")
		return nil, errs.ErrJWT
	}
	logger := log.Logger.With(zap.String("userID", claims.UserID))

	problemID, err := primitive.ObjectIDFromHex(req.Id)
	if err != nil {
		return nil, errs.ErrInvalidID
	}

	teamID, err := primitive.ObjectIDFromHex(claims.Team)
	if err != nil {
		return nil, errs.ErrJWT
	}

	if req.Delete {
		_, err = h.cSolutions.DeleteOne(ctx, bson.M{"problem_id": problemID, "team": teamID})
		if err != nil {
			logger.Error("database error", zap.Error(err))
			return nil, errs.ErrDatabase
		}

		err := events.PublishSolution(&events.SolutionEvent{
			Type:      events.SDelete,
			ProblemID: problemID,
			Team:      teamID,
		})
		if err != nil {
			return nil, errs.ErrQueue
		}

		return res, nil
	}

	s := &entity.Solution{
		Team:      teamID,
		ProblemID: problemID,
		Value:     req.Value,
	}

	_, err = h.cSolutions.UpdateOne(ctx, bson.M{"problem_id": problemID, "team": teamID}, bson.M{"$set": s}, options.Update().SetUpsert(true))
	if err != nil {
		logger.Error("database error", zap.Error(err))
		return nil, errs.ErrDatabase
	}

	err = events.PublishSolution(&events.SolutionEvent{
		Type:      events.SChange,
		ProblemID: problemID,
		Team:      teamID,
		Value:     req.Value,
	})
	if err != nil {
		logger.Error("queue error", zap.Error(err))
		return nil, errs.ErrQueue
	}

	historyEntry := &entity.History{
		Team:      teamID,
		ProblemID: problemID,
		Value:     req.Value,
		Time:      time.Now(),
	}

	_, err = h.cHistory.InsertOne(context.Background(), historyEntry)
	if err != nil {
		logger.Error("failed to insert historical data", zap.Error(err))
	}

	return res, nil
}
func (h *competitionHandler) GetTimes(req *pb.GetTimesRequest, stream pb.Competition_GetTimesServer) error {
	logger := log.Logger

	ch, err := events.ConsumeTime(stream.Context())
	if err != nil {
		logger.Error("queue error", zap.Error(err))
		return errs.ErrQueue
	}

	t := &entity.Info{}
	err = h.cInfo.FindOne(stream.Context(), bson.M{}).Decode(t)
	if err != nil {
		logger.Error("database error", zap.Error(err))
		return errs.ErrDatabase
	}

	err = stream.Send(&pb.GetTimesResponse{
		Start: t.Time.StartDate.Format(time.RFC3339),
		End:   t.Time.EndDate.Format(time.RFC3339),
	})
	if err != nil {
		logger.Debug("sending failed", zap.Error(err))
		return err
	}

L1:
	for {
		select {
		case <-stream.Context().Done():
			break L1
		case t := <-ch:
			err = stream.Send(&pb.GetTimesResponse{
				Start: t.Start.Format(time.RFC3339),
				End:   t.End.Format(time.RFC3339),
			})
			if err != nil {
				logger.Debug("sending failed", zap.Error(err))
				return err
			}
		}
	}

	return nil
}

func NewCompetitionHandler(client *mongo.Client) *competitionHandler {
	return &competitionHandler{
		cSolutions: client.Database("comp").Collection("solutions"),
		cProblems:  client.Database("comp").Collection("problems"),
		cInfo:      client.Database("comp").Collection("info"),
		cHistory:   client.Database("comp").Collection("history"),
		jwt:        jwt.NewJWT(client, []byte("test-key")),
	}
}
