package handler

import (
	"context"
	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/mongo"
	"go.uber.org/zap"
	"google.golang.org/grpc/metadata"
	"matverseny-backend/entity"
	"matverseny-backend/errs"
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
	md, ok := metadata.FromIncomingContext(stream.Context())
	if !ok {
		return errs.ErrUnauthorized
	}

	claims, err := jwt.ValidateAccessToken(md, h.key)
	if err != nil {
		if err == jwt.ErrExpired {
			return errs.ErrTokenExpired
		}
		if err == errs.ErrUnauthorized {
			return err
		}

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

	cs, err := h.cProblems.Watch(stream.Context(), mongo.Pipeline{})
	if err != nil {
		logger.Error("failed to watch database", zap.Error(err))
		return errs.ErrDatabase
	}
	defer cs.Close(context.Background())

	for cs.Next(stream.Context()) {
		data := &bson.M{}
		err := cs.Decode(data)
		if err != nil {
			logger.Error("decode error", zap.Error(err))
			return errs.ErrDatabase
		}
		logger.Debug("", zap.Any("data", data))
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
	return &competitionHandler{
		cSolutions: client.Database("comp").Collection("solutions"),
		cProblems:  client.Database("comp").Collection("problems"),
		cTime:      client.Database("comp").Collection("time"),
		key:        []byte("test-key"),
	}
}
