package handler

import (
	"context"
	"errors"
	"go.mongodb.org/mongo-driver/mongo"
	pb "matverseny-backend/proto"
)

type competitionHandler struct {
	cSolution *mongo.Collection

	pb.UnimplementedCompetitionServer
}

func (h *competitionHandler) GetSolutions(req *pb.GetSolutionsRequest, stream pb.Competition_GetSolutionsServer) error {
	return errors.New("not implemented")
}

func (h *competitionHandler) SetSolutions(ctx context.Context, req *pb.SetSolutionsRequest) (*pb.SetSolutionsResponse, error) {
	return nil, errors.New("not implemented")
}

func NewCompetitionHandler(client *mongo.Client) *competitionHandler {
	return &competitionHandler{
		cSolution: client.Database("comp").Collection("solutions"),
	}
}
