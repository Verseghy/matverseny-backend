package handler

import (
	"context"
	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/mongo"
	"go.uber.org/zap"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"matverseny-backend/entity"
	"matverseny-backend/errs"
	"matverseny-backend/events"
	"matverseny-backend/jwt"
	"matverseny-backend/log"
	pb "matverseny-backend/proto"
	"sync"
	"time"
)

type superAdminHandler struct {
	cInfo *mongo.Collection
	m     sync.Mutex
	jwt   jwt.JWT

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
	_, err = h.cInfo.UpdateOne(ctx, bson.M{}, bson.M{"$set": bson.M{"time": t}})
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
func (h *superAdminHandler) GetResults(req *pb.GetResultsRequest, stream pb.SuperAdmin_GetResultsServer) error {
	return status.Errorf(codes.Unimplemented, "method GetResults not implemented")
}

func NewSuperAdminHandler(client *mongo.Client) *superAdminHandler {
	return &superAdminHandler{
		cInfo: client.Database("comp").Collection("info"),
		jwt:   jwt.NewJWT(client, []byte("test-key")),
	}
}
