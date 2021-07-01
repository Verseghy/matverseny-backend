package handler

import (
	"context"
	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/mongo"
	"go.uber.org/zap"
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

	pb.UnimplementedSuperAdminServer
}

func (h *superAdminHandler) AuthFuncOverride(ctx context.Context, fullMethodName string) (context.Context, error) {
	f := jwt.ValidateSuperAdminToken([]byte("test-key"))
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

func NewSuperAdminHandler(client *mongo.Client) *superAdminHandler {
	return &superAdminHandler{
		cInfo: client.Database("comp").Collection("info"),
	}
}
