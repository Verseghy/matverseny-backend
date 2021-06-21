package handler

import (
	"context"
	"go.mongodb.org/mongo-driver/mongo"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"matverseny-backend/errs"
	"matverseny-backend/jwt"
	pb "matverseny-backend/proto"
)

type teamHandler struct {
	cTeams *mongo.Collection
	key    []byte

	pb.UnimplementedTeamServer
}

func (h *teamHandler) AuthFuncOverride(ctx context.Context, fullMethodName string) (context.Context, error) {
	allowedWithoutAuthentication := []string{
		"/competition.Competition/GetTime",
	}

	for _, v := range allowedWithoutAuthentication {
		if v == fullMethodName {
			return ctx, nil
		}
	}

	f := jwt.ValidateAccessToken([]byte("test-key"))
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

	// TODO: check time

	return ctx, nil
}

func (h *teamHandler) CreateTeam(context.Context, *pb.CreateTeamRequest) (*pb.CreateTeamResponse, error) {
	return nil, status.Errorf(codes.Unimplemented, "method CreateTeam not implemented")
}

func (h *teamHandler) JoinTeam(context.Context, *pb.JoinTeamRequest) (*pb.JoinTeamResponse, error) {
	return nil, status.Errorf(codes.Unimplemented, "method JoinTeam not implemented")
}

func (h *teamHandler) LeaveTeam(context.Context, *pb.LeaveTeamRequest) (*pb.LeaveTeamResponse, error) {
	return nil, status.Errorf(codes.Unimplemented, "method LeaveTeam not implemented")
}

func (h *teamHandler) ListMembers(context.Context, *pb.ListMembersRequest) (*pb.ListMembersResponse, error) {
	return nil, status.Errorf(codes.Unimplemented, "method ListMembers not implemented")
}

func (h *teamHandler) UpdateTeam(context.Context, *pb.UpdateTeamRequest) (*pb.UpdateTeamResponse, error) {
	return nil, status.Errorf(codes.Unimplemented, "method UpdateTeam not implemented")
}

func (h *teamHandler) DisbandTeam(context.Context, *pb.DisbandTeamRequest) (*pb.DisbandTeamResponse, error) {
	return nil, status.Errorf(codes.Unimplemented, "method DisbandTeam not implemented")
}

func (h *teamHandler) ChangeLock(context.Context, *pb.ChangeLockRequest) (*pb.ChangeLockResponse, error) {
	return nil, status.Errorf(codes.Unimplemented, "method ChangeLock not implemented")
}

func (h *teamHandler) ChangeCoOwnerStatus(context.Context, *pb.ChangeCoOwnerStatusRequest) (*pb.ChangeCoOwnerStatusResponse, error) {
	return nil, status.Errorf(codes.Unimplemented, "method ChangeCoOwnerStatus not implemented")
}

func (h *teamHandler) KickUser(context.Context, *pb.KickUserRequest) (*pb.KickUserResponse, error) {
	return nil, status.Errorf(codes.Unimplemented, "method KickUser not implemented")
}

func (h *teamHandler) GenerateJoinCode(context.Context, *pb.GenerateJoinCodeRequest) (*pb.GenerateJoinCodeResponse, error) {
	return nil, status.Errorf(codes.Unimplemented, "method GenerateJoinCode not implemented")
}

func NewTeamHandler(client *mongo.Client) *teamHandler {
	return &teamHandler{
		cTeams: client.Database("comp").Collection("teams"),
		key:    []byte("test-key"),
	}
}
