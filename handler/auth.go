package handler

import (
	"context"
	"errors"
	"go.mongodb.org/mongo-driver/mongo"
	pb "matverseny-backend/proto"
)

type authHandler struct {
	c *mongo.Collection

	pb.UnimplementedAuthServer
}

func (h *authHandler) Login(ctx context.Context, req *pb.LoginRequest) (*pb.LoginResponse, error) {
	return nil, errors.New("not implemented")
}

func (h *authHandler) Register(ctx context.Context, req *pb.RegisterRequest) (*pb.RegisterResponse, error) {
	return nil, errors.New("not implemented")
}

func NewAuthHandler(client *mongo.Client) *authHandler {
	return &authHandler{
		c: client.Database("comp").Collection("auth"),
	}
}
