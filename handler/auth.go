package handler

import (
	"context"
	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/bson/primitive"
	"go.mongodb.org/mongo-driver/mongo"
	"go.mongodb.org/mongo-driver/mongo/options"
	"go.mongodb.org/mongo-driver/x/bsonx"
	"go.uber.org/zap"
	"golang.org/x/crypto/bcrypt"
	"matverseny-backend/entity"
	"matverseny-backend/errs"
	"matverseny-backend/jwt"
	"matverseny-backend/log"
	pb "matverseny-backend/proto"
	"net/mail"
)

type authHandler struct {
	key []byte
	c   *mongo.Collection

	pb.UnimplementedAuthServer
}

func (h *authHandler) Login(ctx context.Context, req *pb.LoginRequest) (*pb.LoginResponse, error) {
	res := &pb.LoginResponse{}

	if req.Email == "" {
		return nil, errs.ErrEmailRequired
	}

	if req.Password == "" {
		return nil, errs.ErrPasswordRequired
	}

	u := &entity.User{}
	err := h.c.FindOne(ctx, bson.M{"email": req.Email}).Decode(u)
	if err != nil {
		if err == mongo.ErrNoDocuments {
			return nil, errs.ErrInvalidEmailOrPassword
		}

		log.Logger.Error("database error", zap.Error(err), zap.String("email", req.Email))
		return nil, errs.ErrDatabase
	}

	err = bcrypt.CompareHashAndPassword([]byte(u.Password), []byte(req.Password))
	if err != nil {
		if err == bcrypt.ErrMismatchedHashAndPassword {
			log.Logger.Debug("invalid password", zap.Error(err))
			return nil, errs.ErrInvalidEmailOrPassword
		}

		return nil, errs.ErrCryptographic
	}

	res.RefreshToken, err = jwt.NewRefreshToken(u, h.key)
	if err != nil {
		log.Logger.Error("jwt failure", zap.Error(err))
		return nil, errs.ErrJWT
	}

	res.AccessToken, err = jwt.NewAccessToken(u, h.key)
	if err != nil {
		log.Logger.Error("jwt failure", zap.Error(err))
		return nil, errs.ErrJWT
	}

	return res, nil
}

func (h *authHandler) Register(ctx context.Context, req *pb.RegisterRequest) (*pb.RegisterResponse, error) {
	res := &pb.RegisterResponse{}

	if req.Name == "" {
		return nil, errs.ErrNameRequired
	}

	if _, err := mail.ParseAddress(req.Email); err != nil {
		return nil, errs.ErrEmailAddressFormat
	}

	if req.Password == "" {
		return nil, errs.ErrPasswordRequired
	}

	if req.School == "" {
		return nil, errs.ErrSchoolRequired
	}

	hash, err := bcrypt.GenerateFromPassword([]byte(req.Password), 10)
	if err != nil {
		log.Logger.Error("failed to generate bcrypt hash", zap.Error(err))
		return nil, errs.ErrCryptographic
	}

	u := &entity.User{
		ID:       primitive.NewObjectID(),
		Email:    req.Email,
		Password: string(hash),
		Name:     req.Name,
		School:   req.School,
		Class:    uint32(req.Class),
	}

	_, err = h.c.InsertOne(ctx, u)
	if err != nil {
		if mongo.IsDuplicateKeyError(err) {
			log.Logger.Debug("already has account", zap.String("email", req.Email), zap.Error(err))
			return nil, errs.ErrAlreadyExists
		}

		log.Logger.Error("failed inserting new user", zap.Error(err))
		return nil, errs.ErrDatabase
	}

	res.RefreshToken, err = jwt.NewRefreshToken(u, h.key)
	if err != nil {
		log.Logger.Error("jwt failure", zap.Error(err))
		return nil, errs.ErrJWT
	}

	res.AccessToken, err = jwt.NewAccessToken(u, h.key)
	if err != nil {
		log.Logger.Error("jwt failure", zap.Error(err))
		return nil, errs.ErrJWT
	}

	return res, nil
}

func (h *authHandler) ForgotPassword(context.Context, *pb.ForgotPasswordRequest) (*pb.ForgotPasswordResponse, error) {
	return nil, errs.ErrNotImplemented
}

func (h *authHandler) RefreshToken(ctx context.Context, req *pb.RefreshTokenRequest) (*pb.RefreshTokenResponse, error) {
	res := &pb.RefreshTokenResponse{}

	claims, err := jwt.ValidateRefreshToken(req.Token, h.key)
	if err != nil {
		if err == jwt.ErrExpired {
			return nil, errs.ErrTokenExpired
		}

		return nil, errs.ErrJWT
	}

	u := &entity.User{}
	id, err := primitive.ObjectIDFromHex(claims.UserID)
	if err != nil {
		log.Logger.Error("failed mongo id", zap.Error(err))
		return nil, errs.ErrJWT
	}

	err = h.c.FindOne(ctx, bson.M{"_id": id}).Decode(u)
	if err != nil {
		if err == mongo.ErrNoDocuments {
			return nil, errs.ErrJWT
		}

		log.Logger.Error("database error", zap.Error(err), zap.String("id", claims.UserID))
		return nil, errs.ErrDatabase
	}

	res.Token, err = jwt.NewAccessToken(u, h.key)
	if err != nil {
		log.Logger.Error("jwt failure", zap.Error(err))
		return nil, errs.ErrJWT
	}

	return res, nil
}

func NewAuthHandler(client *mongo.Client) *authHandler {
	_, err := client.Database("comp").Collection("auth").Indexes().CreateMany(context.Background(), []mongo.IndexModel{
		{Keys: bsonx.Doc{{Key: "email", Value: bsonx.Int32(1)}}, Options: options.Index().SetUnique(true)},
	})
	if err != nil {
		log.Logger.Fatal("unable to create index", zap.Error(err))
	}

	return &authHandler{
		key: []byte("test-key"),
		c:   client.Database("comp").Collection("auth"),
	}
}
