package handler

import (
	"context"
	"go.mongodb.org/mongo-driver/bson"
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

type competitionHandler struct {
	cSolutions *mongo.Collection
	cProblems  *mongo.Collection
	cTime      *mongo.Collection
	key        []byte

	pb.UnimplementedCompetitionServer
}

func (h *competitionHandler) GetProblems(req *pb.GetProblemsRequest, stream pb.Competition_GetProblemsServer) error {
	claims, ok := jwt.GetClaimsFromCtx(stream.Context())
	if !ok {
		log.Logger.Error("jwt had no data")
		return errs.ErrJWT
	}
	logger := log.Logger.With(zap.String("userID", claims.UserID))

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

	ch := events.ConsumeProblem(stream.Context())

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
					if p.Type != events.PSwap {
						return nil
					}

					return &pb.ProblemStream_Create{At: p.At}
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
func (h *competitionHandler) GetSolutions(*pb.GetSolutionsRequest, pb.Competition_GetSolutionsServer) error {
	return errs.ErrNotImplemented
}
func (h *competitionHandler) SetSolutions(context.Context, *pb.SetSolutionsRequest) (*pb.SetSolutionsResponse, error) {
	return nil, errs.ErrNotImplemented
}
func (h *competitionHandler) GetTimes(*pb.GetTimesRequest, pb.Competition_GetTimesServer) error {
	return errs.ErrNotImplemented
}

func NewCompetitionHandler(client *mongo.Client) *competitionHandler {
	_, err := client.Database("comp").Collection("problems").Indexes().CreateMany(context.Background(), []mongo.IndexModel{
		{Keys: bsonx.Doc{{Key: "position", Value: bsonx.Int32(1)}}, Options: options.Index().SetUnique(true)},
	})
	if err != nil {
		log.Logger.Fatal("unable to create index", zap.Error(err))
	}

	return &competitionHandler{
		cSolutions: client.Database("comp").Collection("solutions"),
		cProblems:  client.Database("comp").Collection("problems"),
		cTime:      client.Database("comp").Collection("time"),
		key:        []byte("test-key"),
	}
}
