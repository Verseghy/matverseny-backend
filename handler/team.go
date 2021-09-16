package handler

import (
	"context"
	"crypto/rand"
	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/bson/primitive"
	"go.mongodb.org/mongo-driver/mongo"
	"go.uber.org/zap"
	"io"
	"matverseny-backend/entity"
	"matverseny-backend/errs"
	"matverseny-backend/jwt"
	"matverseny-backend/log"
	pb "matverseny-backend/proto"
	"sync"
)

type teamHandler struct {
	cTeams *mongo.Collection
	cUser  *mongo.Collection
	jwt    jwt.JWT
	m      sync.Mutex

	pb.UnimplementedTeamServer
}

var table2 = []byte{
	'0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
	'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J',
	'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T',
	'U', 'V', 'W', 'X', 'Y', 'Z',
}

func mustGenerateNewCode() string {
	b := make([]byte, 6)
	n, err := io.ReadAtLeast(rand.Reader, b, 6)
	if n != 6 || err != nil {
		log.Logger.Fatal("can't read rand", zap.Error(err))
	}

	for i := 0; i < 6; i++ {
		b[i] = table2[int(b[i])%len(table2)]
	}

	return string(b)
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

	f := h.jwt.ValidateAccessToken()
	ctx, err := f(ctx)
	if err != nil {
		return nil, err
	}

	_, ok := jwt.GetClaimsFromCtx(ctx)
	if !ok {
		return nil, errs.ErrJWT
	}

	// TODO: check time

	return ctx, nil
}

func (h *teamHandler) CreateTeam(ctx context.Context, req *pb.CreateTeamRequest) (*pb.CreateTeamResponse, error) {
	h.m.Lock()
	defer h.m.Unlock()

	res := &pb.CreateTeamResponse{}

	claims, ok := jwt.GetClaimsFromCtx(ctx)
	if !ok {
		log.Logger.Error("jwt had no data")
		return nil, errs.ErrJWT
	}
	logger := log.Logger.With(zap.String("userID", claims.UserID))

	userID, err := primitive.ObjectIDFromHex(claims.UserID)
	if err != nil {
		logger.Error("invalid user id", zap.Error(err))
		return nil, errs.ErrJWT
	}

	// validate
	{
		if claims.Team != "" {
			logger.Info("user already has team")
			return nil, errs.ErrHasTeam
		}

		if len(req.Name) > 64 {
			logger.Info("team name longer than 64 characters")
			return nil, errs.ErrTeamNameTooLong
		}

		err := h.cTeams.FindOne(ctx, bson.M{"members": userID}).Err()
		if err == nil {
			logger.Info("user already has team")
			return nil, errs.ErrHasTeam
		}
		if err != mongo.ErrNoDocuments {
			logger.Error("database error")
			return nil, errs.ErrDatabase
		}

		err = h.cTeams.FindOne(ctx, bson.M{"team_name": req.Name}).Err()
		if err == nil {
			logger.Info("team name already exists")
			return nil, errs.ErrTeamNameTaken
		}
		if err != mongo.ErrNoDocuments {
			logger.Error("database error", zap.Error(err))
			return nil, errs.ErrDatabase
		}
	}

	var joinCode string
	var stop bool
	for i := 0; i < 10; i++ {
		joinCode = mustGenerateNewCode()
		err = h.cTeams.FindOne(ctx, bson.M{"join_code": joinCode}).Err()
		if err == nil {
			continue
		}
		if err != mongo.ErrNoDocuments {
			logger.Error("database error", zap.Error(err))
			return nil, errs.ErrDatabase
		}
		stop = true
		break
	}
	if !stop {
		logger.Error("couldn't generate new code")
		return nil, errs.ErrWTF
	}

	t := &entity.Team{
		TeamName: req.Name,
		Members:  []primitive.ObjectID{userID},
		Owner:    userID,
		CoOwner:  nil,
		Locked:   false,
		JoinCode: joinCode,
	}
	_, err = h.cTeams.InsertOne(ctx, t)
	if err != nil {
		logger.Error("database error", zap.Error(err))
		return nil, errs.ErrDatabase
	}

	return res, nil
}

func (h *teamHandler) JoinTeam(ctx context.Context, req *pb.JoinTeamRequest) (*pb.JoinTeamResponse, error) {
	h.m.Lock()
	defer h.m.Unlock()

	res := &pb.JoinTeamResponse{}

	claims, ok := jwt.GetClaimsFromCtx(ctx)
	if !ok {
		log.Logger.Error("jwt had no data")
		return nil, errs.ErrJWT
	}
	logger := log.Logger.With(zap.String("userID", claims.UserID))

	userID, err := primitive.ObjectIDFromHex(claims.UserID)
	if err != nil {
		logger.Error("invalid user id", zap.Error(err))
		return nil, errs.ErrJWT
	}

	// validate
	{
		if claims.Team != "" {
			logger.Info("user already has team")
			return nil, errs.ErrHasTeam
		}

		err := h.cTeams.FindOne(ctx, bson.M{"members": userID}).Err()
		if err == nil {
			logger.Info("user already has team")
			return nil, errs.ErrHasTeam
		}
		if err != mongo.ErrNoDocuments {
			logger.Error("database error")
			return nil, errs.ErrDatabase
		}
	}

	t := &entity.Team{}
	err = h.cTeams.FindOne(ctx, bson.M{"join_code": req.Code}).Decode(t)
	if err != nil {
		if err == mongo.ErrNoDocuments {
			return nil, errs.ErrNotFound
		}

		logger.Error("database error", zap.Error(err))
		return nil, errs.ErrDatabase
	}

	t.Members = append(t.Members, userID)
	_, err = h.cTeams.ReplaceOne(ctx, bson.M{"_id": t.ID}, t)
	if err != nil {
		logger.Error("database error", zap.Error(err))
		return nil, errs.ErrDatabase
	}

	return res, nil
}

func (h *teamHandler) LeaveTeam(ctx context.Context, req *pb.LeaveTeamRequest) (*pb.LeaveTeamResponse, error) {
	h.m.Lock()
	defer h.m.Unlock()

	res := &pb.LeaveTeamResponse{}

	claims, ok := jwt.GetClaimsFromCtx(ctx)
	if !ok {
		log.Logger.Error("jwt had no data")
		return nil, errs.ErrJWT
	}
	logger := log.Logger.With(zap.String("userID", claims.UserID))

	userID, err := primitive.ObjectIDFromHex(claims.UserID)
	if err != nil {
		logger.Error("invalid user id", zap.Error(err))
		return nil, errs.ErrJWT
	}

	t := &entity.Team{}
	err = h.cTeams.FindOne(ctx, bson.M{"members": userID}).Decode(t)
	if err != nil {
		if err == mongo.ErrNoDocuments {
			return nil, errs.ErrNoTeam
		}

		logger.Error("database error", zap.Error(err))
		return nil, errs.ErrDatabase
	}

	if t.Owner == userID {
		logger.Info("owner tried leaving")
		return nil, errs.ErrOwnerCantLeave
	}

	mems := make([]primitive.ObjectID, 0, len(t.Members))
	for _, v := range t.Members {
		if v != userID {
			mems = append(mems, v)
		}
	}
	t.Members = mems

	if t.CoOwner != nil && *t.CoOwner == userID {
		t.CoOwner = nil
	}

	_, err = h.cTeams.ReplaceOne(ctx, bson.M{"_id": t.ID}, t)
	if err != nil {
		logger.Error("database error", zap.Error(err))
		return nil, errs.ErrDatabase
	}

	return res, nil
}

func (h *teamHandler) GetTeamInfo(ctx context.Context, req *pb.GetTeamInfoRequest) (*pb.GetTeamInfoResponse, error) {
	h.m.Lock()
	defer h.m.Unlock()

	res := &pb.GetTeamInfoResponse{}

	claims, ok := jwt.GetClaimsFromCtx(ctx)
	if !ok {
		log.Logger.Error("jwt had no data")
		return nil, errs.ErrJWT
	}
	logger := log.Logger.With(zap.String("userID", claims.UserID))

	userID, err := primitive.ObjectIDFromHex(claims.UserID)
	if err != nil {
		logger.Error("invalid user id", zap.Error(err))
		return nil, errs.ErrJWT
	}

	t := &entity.Team{}
	err = h.cTeams.FindOne(ctx, bson.M{"members": userID}).Decode(t)
	if err != nil {
		if err == mongo.ErrNoDocuments {
			return nil, errs.ErrNoTeam
		}

		logger.Error("database error", zap.Error(err))
		return nil, errs.ErrDatabase
	}

	res.JoinCode = t.JoinCode
	res.Name = t.TeamName

	mems := make([]*pb.GetTeamInfoResponse_Member, 0, len(t.Members))
	for _, v := range t.Members {
		u := &entity.User{}
		err := h.cUser.FindOne(ctx, bson.M{"_id": v}).Decode(u)
		if err != nil {
			log.Logger.Error("database error", zap.Error(err))
			return nil, errs.ErrDatabase
		}

		mems = append(mems, &pb.GetTeamInfoResponse_Member{
			ID:    u.ID.Hex(),
			Name:  u.Name,
			Class: pb.GetTeamInfoResponse_Member_Class(u.Class),
			Rank: func() pb.GetTeamInfoResponse_Member_Rank {
				if u.ID == t.Owner {
					return pb.GetTeamInfoResponse_Member_k_OWNER
				}

				if t.CoOwner != nil && u.ID == *t.CoOwner {
					return pb.GetTeamInfoResponse_Member_k_COOWNER
				}

				return pb.GetTeamInfoResponse_Member_k_MEMBER
			}(),
		})
	}

	res.Members = mems

	return res, nil
}

func (h *teamHandler) UpdateTeam(ctx context.Context, req *pb.UpdateTeamRequest) (*pb.UpdateTeamResponse, error) {
	h.m.Lock()
	defer h.m.Unlock()

	res := &pb.UpdateTeamResponse{}

	claims, ok := jwt.GetClaimsFromCtx(ctx)
	if !ok {
		log.Logger.Error("jwt had no data")
		return nil, errs.ErrJWT
	}
	logger := log.Logger.With(zap.String("userID", claims.UserID))

	userID, err := primitive.ObjectIDFromHex(claims.UserID)
	if err != nil {
		logger.Error("invalid user id", zap.Error(err))
		return nil, errs.ErrJWT
	}

	t := &entity.Team{}
	err = h.cTeams.FindOne(ctx, bson.M{"members": userID}).Decode(t)
	if err != nil {
		if err == mongo.ErrNoDocuments {
			return nil, errs.ErrNoTeam
		}

		logger.Error("database error", zap.Error(err))
		return nil, errs.ErrDatabase
	}

	if t.Owner != userID || (t.CoOwner != nil && *t.CoOwner != userID) {
		logger.Info("not authorized")
		return nil, errs.ErrNotAuthorized
	}

	err = h.cTeams.FindOne(ctx, bson.M{"team_name": req.Name}).Err()
	if err == nil {
		logger.Info("team name already exists")
		return nil, errs.ErrTeamNameTaken
	}
	if err != mongo.ErrNoDocuments {
		logger.Error("database error", zap.Error(err))
		return nil, errs.ErrDatabase
	}

	t.TeamName = req.Name
	_, err = h.cTeams.ReplaceOne(ctx, bson.M{"_id": t.ID}, t)
	if err != nil {
		logger.Error("database error", zap.Error(err))
		return nil, errs.ErrDatabase
	}

	return res, nil
}

func (h *teamHandler) DisbandTeam(ctx context.Context, req *pb.DisbandTeamRequest) (*pb.DisbandTeamResponse, error) {
	h.m.Lock()
	defer h.m.Unlock()

	res := &pb.DisbandTeamResponse{}

	claims, ok := jwt.GetClaimsFromCtx(ctx)
	if !ok {
		log.Logger.Error("jwt had no data")
		return nil, errs.ErrJWT
	}
	logger := log.Logger.With(zap.String("userID", claims.UserID))

	userID, err := primitive.ObjectIDFromHex(claims.UserID)
	if err != nil {
		logger.Error("invalid user id", zap.Error(err))
		return nil, errs.ErrJWT
	}

	t := &entity.Team{}
	err = h.cTeams.FindOne(ctx, bson.M{"members": userID}).Decode(t)
	if err != nil {
		if err == mongo.ErrNoDocuments {
			return nil, errs.ErrNoTeam
		}

		logger.Error("database error", zap.Error(err))
		return nil, errs.ErrDatabase
	}

	if t.Owner != userID {
		logger.Info("not authorized")
		return nil, errs.ErrNotAuthorized
	}

	if len(t.Members) > 1 {
		logger.Info("can't disband non-empty team")
		return nil, errs.ErrDisbandNonEmptyTeam
	}

	_, err = h.cTeams.DeleteOne(ctx, bson.M{"_id": t.ID})
	if err != nil {
		logger.Error("database error", zap.Error(err))
		return nil, errs.ErrDatabase
	}

	return res, nil
}

func (h *teamHandler) ChangeLock(ctx context.Context, req *pb.ChangeLockRequest) (*pb.ChangeLockResponse, error) {
	h.m.Lock()
	defer h.m.Unlock()

	res := &pb.ChangeLockResponse{}

	claims, ok := jwt.GetClaimsFromCtx(ctx)
	if !ok {
		log.Logger.Error("jwt had no data")
		return nil, errs.ErrJWT
	}
	logger := log.Logger.With(zap.String("userID", claims.UserID))

	userID, err := primitive.ObjectIDFromHex(claims.UserID)
	if err != nil {
		logger.Error("invalid user id", zap.Error(err))
		return nil, errs.ErrJWT
	}

	t := &entity.Team{}
	err = h.cTeams.FindOne(ctx, bson.M{"members": userID}).Decode(t)
	if err != nil {
		if err == mongo.ErrNoDocuments {
			return nil, errs.ErrNoTeam
		}

		logger.Error("database error", zap.Error(err))
		return nil, errs.ErrDatabase
	}

	if t.Owner != userID || (t.CoOwner != nil && *t.CoOwner != userID) {
		logger.Info("not authorized")
		return nil, errs.ErrNotAuthorized
	}

	t.Locked = req.ShouldLock
	_, err = h.cTeams.ReplaceOne(ctx, bson.M{"_id": t.ID}, t)
	if err != nil {
		logger.Error("database error", zap.Error(err))
		return nil, errs.ErrDatabase
	}

	return res, nil
}

func (h *teamHandler) ChangeCoOwnerStatus(ctx context.Context, req *pb.ChangeCoOwnerStatusRequest) (*pb.ChangeCoOwnerStatusResponse, error) {
	h.m.Lock()
	defer h.m.Unlock()

	res := &pb.ChangeCoOwnerStatusResponse{}

	claims, ok := jwt.GetClaimsFromCtx(ctx)
	if !ok {
		log.Logger.Error("jwt had no data")
		return nil, errs.ErrJWT
	}
	logger := log.Logger.With(zap.String("userID", claims.UserID))

	userID, err := primitive.ObjectIDFromHex(claims.UserID)
	if err != nil {
		logger.Error("invalid user id", zap.Error(err))
		return nil, errs.ErrJWT
	}

	coownerID, err := primitive.ObjectIDFromHex(req.UserId)
	if err != nil {
		logger.Error("invalid user id", zap.Error(err))
		return nil, errs.ErrJWT
	}

	t := &entity.Team{}
	err = h.cTeams.FindOne(ctx, bson.M{"members": userID}).Decode(t)
	if err != nil {
		if err == mongo.ErrNoDocuments {
			return nil, errs.ErrNoTeam
		}

		logger.Error("database error", zap.Error(err))
		return nil, errs.ErrDatabase
	}

	if t.Owner != userID {
		logger.Info("not authorized")
		return nil, errs.ErrNotAuthorized
	}

	if req.ShouldCoowner {
		t.CoOwner = &coownerID
	} else {
		t.CoOwner = nil
	}
	_, err = h.cTeams.ReplaceOne(ctx, bson.M{"_id": t.ID}, t)
	if err != nil {
		logger.Error("database error", zap.Error(err))
		return nil, errs.ErrDatabase
	}

	return res, nil
}

func (h *teamHandler) KickUser(ctx context.Context, req *pb.KickUserRequest) (*pb.KickUserResponse, error) {
	h.m.Lock()
	defer h.m.Unlock()

	res := &pb.KickUserResponse{}

	claims, ok := jwt.GetClaimsFromCtx(ctx)
	if !ok {
		log.Logger.Error("jwt had no data")
		return nil, errs.ErrJWT
	}
	logger := log.Logger.With(zap.String("userID", claims.UserID))

	userID, err := primitive.ObjectIDFromHex(claims.UserID)
	if err != nil {
		logger.Error("invalid user id", zap.Error(err))
		return nil, errs.ErrJWT
	}

	kickedUser, err := primitive.ObjectIDFromHex(req.UserId)
	if err != nil {
		logger.Error("invalid user id", zap.Error(err))
		return nil, errs.ErrJWT
	}

	t := &entity.Team{}
	err = h.cTeams.FindOne(ctx, bson.M{"members": userID}).Decode(t)
	if err != nil {
		if err == mongo.ErrNoDocuments {
			return nil, errs.ErrNoTeam
		}

		logger.Error("database error", zap.Error(err))
		return nil, errs.ErrDatabase
	}

	if t.Owner == kickedUser {
		logger.Info("not authorized")
		return nil, errs.ErrNotAuthorized
	}

	if t.Owner != userID && (t.CoOwner != nil && *t.CoOwner != userID) {
		logger.Info("not authorized")
		return nil, errs.ErrNotAuthorized
	}

	mems := make([]primitive.ObjectID, 0, len(t.Members))
	for _, v := range t.Members {
		if v != kickedUser {
			mems = append(mems, v)
		}
	}
	t.Members = mems

	if t.CoOwner != nil && *t.CoOwner == userID {
		t.CoOwner = nil
	}

	_, err = h.cTeams.ReplaceOne(ctx, bson.M{"_id": t.ID}, t)
	if err != nil {
		logger.Error("database error", zap.Error(err))
		return nil, errs.ErrDatabase
	}

	return res, nil
}

func (h *teamHandler) GenerateJoinCode(ctx context.Context, req *pb.GenerateJoinCodeRequest) (*pb.GenerateJoinCodeResponse, error) {
	h.m.Lock()
	defer h.m.Unlock()

	res := &pb.GenerateJoinCodeResponse{}

	claims, ok := jwt.GetClaimsFromCtx(ctx)
	if !ok {
		log.Logger.Error("jwt had no data")
		return nil, errs.ErrJWT
	}
	logger := log.Logger.With(zap.String("userID", claims.UserID))

	userID, err := primitive.ObjectIDFromHex(claims.UserID)
	if err != nil {
		logger.Error("invalid user id", zap.Error(err))
		return nil, errs.ErrJWT
	}

	t := &entity.Team{}
	err = h.cTeams.FindOne(ctx, bson.M{"members": userID}).Decode(t)
	if err != nil {
		if err == mongo.ErrNoDocuments {
			return nil, errs.ErrNoTeam
		}

		logger.Error("database error", zap.Error(err))
		return nil, errs.ErrDatabase
	}

	if t.Owner != userID || (t.CoOwner != nil && *t.CoOwner != userID) {
		logger.Info("not authorized")
		return nil, errs.ErrNotAuthorized
	}

	var joinCode string
	var stop bool
	for i := 0; i < 10; i++ {
		joinCode = mustGenerateNewCode()
		err = h.cTeams.FindOne(ctx, bson.M{"join_code": joinCode}).Err()
		if err == nil {
			continue
		}
		if err != mongo.ErrNoDocuments {
			logger.Error("database error", zap.Error(err))
			return nil, errs.ErrDatabase
		}
		stop = true
		break
	}
	if !stop {
		logger.Error("couldn't generate new code")
		return nil, errs.ErrWTF
	}
	t.JoinCode = joinCode
	_, err = h.cTeams.ReplaceOne(ctx, bson.M{"_id": t.ID}, t)
	if err != nil {
		logger.Error("database error", zap.Error(err))
		return nil, errs.ErrDatabase
	}

	res.NewCode = joinCode

	return res, nil
}

func NewTeamHandler(client *mongo.Client) *teamHandler {
	return &teamHandler{
		cTeams: client.Database("comp").Collection("teams"),
		cUser:  client.Database("comp").Collection("auth"),
		jwt:    jwt.NewJWT(client, []byte("test-key")),
	}
}
