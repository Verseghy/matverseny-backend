package handler

import (
	"context"
	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/bson/primitive"
	"go.mongodb.org/mongo-driver/mongo"
	"go.uber.org/zap"
	"matverseny-backend/entity"
	"matverseny-backend/errs"
	"matverseny-backend/events"
	"matverseny-backend/jwt"
	"matverseny-backend/log"
	pb "matverseny-backend/proto"
)

type adminHandler struct {
	cProblems *mongo.Collection
	cTime     *mongo.Collection

	pb.UnimplementedAdminServer
}

func (h *adminHandler) AuthFuncOverride(ctx context.Context, fullMethodName string) (context.Context, error) {
	f := jwt.ValidateAccessToken([]byte("test-key"))
	ctx, err := f(ctx)
	if err != nil {
		return nil, err
	}

	claims, ok := jwt.GetClaimsFromCtx(ctx)
	if !ok {
		return nil, errs.ErrJWT
	}

	if !claims.IsAdmin {
		return nil, errs.ErrNotAdmin
	}

	return ctx, nil
}

func (h *adminHandler) CreateProblem(ctx context.Context, req *pb.CreateRequest) (*pb.CreateResponse, error) {
	res := &pb.CreateResponse{}

	claims, ok := jwt.GetClaimsFromCtx(ctx)
	if !ok {
		log.Logger.Error("jwt had no data")
		return nil, errs.ErrJWT
	}
	logger := log.Logger.With(zap.String("userID", claims.UserID))

	if req.At < 0 {
		return nil, errs.ErrInvalidPosition
	}

	_, err := h.cProblems.UpdateMany(ctx, bson.M{"position": bson.M{"$gte": req.At}}, bson.M{"$inc": bson.M{"position": 1}})
	if err != nil {
		logger.Error("database error", zap.Error(err))
		return nil, errs.ErrDatabase
	}

	_, err = h.cProblems.InsertOne(ctx, &entity.Problem{
		ID:       primitive.NewObjectID(),
		Position: req.At,
	})
	if err != nil {
		logger.Error("database error", zap.Error(err))
		return nil, errs.ErrDatabase
	}

	events.PublishProblem(&events.ProblemEvent{
		Type: events.PCreate,
		At:   req.At,
	})

	return res, nil
}

func (h *adminHandler) GetProblems(req *pb.ProblemStreamRequest, stream pb.Admin_GetProblemsServer) error {
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
					Id:       p.ID.Hex(),
					Body:     p.Body,
					Image:    p.Image,
					Solution: p.Solution,
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

					return &pb.ProblemStream_Update{Problem: p.Problem.ToAdminProto()}
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

func (h *adminHandler) UpdateProblem(ctx context.Context, req *pb.UpdateRequest) (*pb.UpdateResponse, error) {
	res := &pb.UpdateResponse{}

	claims, ok := jwt.GetClaimsFromCtx(ctx)
	if !ok {
		log.Logger.Error("jwt had no data")
		return nil, errs.ErrJWT
	}
	logger := log.Logger.With(zap.String("userID", claims.UserID))

	mongoID, err := primitive.ObjectIDFromHex(req.Problem.Id)
	if err != nil {
		return nil, errs.ErrInvalidID
	}

	p := &entity.Problem{}
	p.FromProto(req.Problem)

	err = h.cProblems.FindOneAndUpdate(ctx, bson.M{"_id": mongoID}, bson.M{"$set": p}).Decode(p)
	if err != nil {
		if err == mongo.ErrNoDocuments {
			return nil, errs.ErrNotFound
		}

		logger.Error("database error", zap.Error(err))
		return nil, errs.ErrDatabase
	}

	events.PublishProblem(&events.ProblemEvent{
		Type:    events.PChange,
		Problem: p,
	})

	return res, nil
}

func (h *adminHandler) DeleteProblem(ctx context.Context, req *pb.DeleteRequest) (*pb.DeleteResponse, error) {
	res := &pb.DeleteResponse{}

	claims, ok := jwt.GetClaimsFromCtx(ctx)
	if !ok {
		log.Logger.Error("jwt had no data")
		return nil, errs.ErrJWT
	}
	logger := log.Logger.With(zap.String("userID", claims.UserID))

	p := &entity.Problem{}
	mongoID, err := primitive.ObjectIDFromHex(req.Id)
	if err != nil {
		return nil, errs.ErrInvalidID
	}

	err = h.cProblems.FindOneAndDelete(ctx, bson.M{"_id": mongoID}).Decode(p)
	if err != nil {
		if err == mongo.ErrNoDocuments {
			return nil, errs.ErrNotFound
		}

		logger.Error("database error", zap.Error(err))
		return nil, errs.ErrDatabase
	}

	_, err = h.cProblems.UpdateMany(ctx, bson.M{"position": bson.M{"$gte": p.Position}}, bson.M{"$inc": bson.M{"position": -1}})
	if err != nil {
		logger.Error("database error", zap.Error(err))
		return nil, errs.ErrDatabase
	}

	events.PublishProblem(&events.ProblemEvent{
		Type:    events.PDelete,
		Problem: p,
	})

	return res, nil
}

func (h *adminHandler) SwapProblem(ctx context.Context, req *pb.SwapRequest) (*pb.SwapResponse, error) {
	res := &pb.SwapResponse{}

	claims, ok := jwt.GetClaimsFromCtx(ctx)
	if !ok {
		log.Logger.Error("jwt had no data")
		return nil, errs.ErrJWT
	}
	logger := log.Logger.With(zap.String("userID", claims.UserID))

	p1 := &entity.Problem{}
	p2 := &entity.Problem{}

	mongoID1, err := primitive.ObjectIDFromHex(req.A)
	if err != nil {
		return nil, errs.ErrInvalidID
	}
	mongoID2, err := primitive.ObjectIDFromHex(req.B)
	if err != nil {
		return nil, errs.ErrInvalidID
	}

	err = h.cProblems.FindOne(ctx, bson.M{"_id": mongoID1}).Decode(p1)
	if err != nil {
		if err == mongo.ErrNoDocuments {
			return nil, errs.ErrNotFound
		}

		logger.Error("database error", zap.Error(err))
		return nil, errs.ErrDatabase
	}

	err = h.cProblems.FindOne(ctx, bson.M{"_id": mongoID2}).Decode(p2)
	if err != nil {
		if err == mongo.ErrNoDocuments {
			return nil, errs.ErrNotFound
		}

		logger.Error("database error", zap.Error(err))
		return nil, errs.ErrDatabase
	}

	_, err = h.cProblems.UpdateByID(ctx, mongoID1, bson.M{"$set": bson.M{"position": p2.Position}})
	if err != nil {
		logger.Error("database error", zap.Error(err))
		return nil, errs.ErrDatabase
	}

	_, err = h.cProblems.UpdateByID(ctx, mongoID2, bson.M{"$set": bson.M{"position": p1.Position}})
	if err != nil {
		logger.Error("database error", zap.Error(err))
		return nil, errs.ErrDatabase
	}

	events.PublishProblem(&events.ProblemEvent{
		Type: events.PSwap,
		A:    p1,
		B:    p2,
	})

	return res, nil
}

func NewAdminHandler(client *mongo.Client) *adminHandler {
	return &adminHandler{
		cProblems: client.Database("comp").Collection("problems"),
		cTime:     client.Database("comp").Collection("time"),
	}
}
