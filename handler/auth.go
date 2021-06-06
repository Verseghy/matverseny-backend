package handler

import (
	"context"
	"errors"
	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/bson/primitive"
	"go.mongodb.org/mongo-driver/mongo"
	"go.mongodb.org/mongo-driver/mongo/options"
	"go.mongodb.org/mongo-driver/x/bsonx"
	"go.uber.org/zap"
	"golang.org/x/crypto/bcrypt"
	"matverseny-backend/entity"
	"matverseny-backend/jwt"
	"matverseny-backend/log"
	pb "matverseny-backend/proto"
	"net/mail"
)

var (
	ErrNotImplemented         = errors.New("E0000: not implemented")
	ErrEmailRequired          = errors.New("E0001: email is required")
	ErrPasswordRequired       = errors.New("E0002: password is required")
	ErrInvalidEmailOrPassword = errors.New("E0003: invalid email or password")
	ErrDatabase               = errors.New("E0004: database error")
	ErrCryptographic          = errors.New("E0005: cryptographic failure")
	ErrJWT                    = errors.New("E0006: JWT failure")
	ErrNameRequired           = errors.New("E0007: name is required")
	ErrEmailAddressFormat     = errors.New("E0008: email address format incorrect")
	ErrSchoolRequired         = errors.New("E0009: school is required")
	ErrAlreadyExists          = errors.New("E0010: user already registered")
	ErrTokenExpired           = errors.New("E0011: token expired")
)

type authHandler struct {
	key []byte
	c   *mongo.Collection

	pb.UnimplementedAuthServer
}

func (h *authHandler) Login(ctx context.Context, req *pb.LoginRequest) (*pb.LoginResponse, error) {
	res := &pb.LoginResponse{}

	if req.Email == "" {
		return nil, ErrEmailRequired
	}

	if req.Password == "" {
		return nil, ErrPasswordRequired
	}

	u := &entity.User{}
	err := h.c.FindOne(ctx, bson.M{"email": req.Email}).Decode(u)
	if err != nil {
		if err == mongo.ErrNoDocuments {
			return nil, ErrInvalidEmailOrPassword
		}

		log.Logger.Error("database error", zap.Error(err), zap.String("email", req.Email))
		return nil, ErrDatabase
	}

	err = bcrypt.CompareHashAndPassword([]byte(u.Password), []byte(req.Password))
	if err != nil {
		if err == bcrypt.ErrMismatchedHashAndPassword {
			log.Logger.Debug("invalid password", zap.Error(err))
		}

		return nil, ErrCryptographic
	}

	res.RefreshToken, err = jwt.NewRefreshToken(u, h.key)
	if err != nil {
		log.Logger.Error("jwt failure", zap.Error(err))
		return nil, ErrJWT
	}

	res.AccessToken, err = jwt.NewAccessToken(u, h.key)
	if err != nil {
		log.Logger.Error("jwt failure", zap.Error(err))
		return nil, ErrJWT
	}

	return res, nil
}

func (h *authHandler) Register(ctx context.Context, req *pb.RegisterRequest) (*pb.RegisterResponse, error) {
	res := &pb.RegisterResponse{}

	if req.Name == "" {
		return nil, ErrNameRequired
	}

	if _, err := mail.ParseAddress(req.Email); err != nil {
		return nil, ErrEmailAddressFormat
	}

	if req.Password == "" {
		return nil, ErrPasswordRequired
	}

	if req.School == "" {
		return nil, ErrSchoolRequired
	}

	hash, err := bcrypt.GenerateFromPassword([]byte(req.Password), 10)
	if err != nil {
		log.Logger.Error("failed to generate bcrypt hash", zap.Error(err))
		return nil, ErrCryptographic
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
			return nil, ErrAlreadyExists
		}

		log.Logger.Error("failed inserting new user", zap.Error(err))
		return nil, ErrDatabase
	}

	res.RefreshToken, err = jwt.NewRefreshToken(u, h.key)
	if err != nil {
		log.Logger.Error("jwt failure", zap.Error(err))
		return nil, ErrJWT
	}

	res.AccessToken, err = jwt.NewAccessToken(u, h.key)
	if err != nil {
		log.Logger.Error("jwt failure", zap.Error(err))
		return nil, ErrJWT
	}

	return res, nil
}

func (h *authHandler) ForgotPassword(context.Context, *pb.ForgotPasswordRequest) (*pb.ForgotPasswordResponse, error) {
	return nil, ErrNotImplemented
}

func (h *authHandler) RefreshToken(ctx context.Context, req *pb.RefreshTokenRequest) (*pb.RefreshTokenResponse, error) {
	res := &pb.RefreshTokenResponse{}

	claims, err := jwt.ValidateRefreshToken(req.Token, h.key)
	if err != nil {
		if err == jwt.ErrExpired {
			return nil, ErrTokenExpired
		}

		return nil, ErrJWT
	}

	u := &entity.User{}
	err = h.c.FindOne(ctx, bson.M{"_id": primitive.ObjectIDFromHex(claims.UserID)}).Decode(u)
	if err != nil {
		if err == mongo.ErrNoDocuments {
			return nil, ErrJWT
		}

		log.Logger.Error("database error", zap.Error(err), zap.String("id", claims.UserID))
		return nil, ErrDatabase
	}

	res.Token, err = jwt.NewAccessToken(u, h.key)
	if err != nil {
		log.Logger.Error("jwt failure", zap.Error(err))
		return nil, ErrJWT
	}

	return res, nil
}

func NewAuthHandler(client *mongo.Client) *authHandler {
	_, err := client.Database("comp").Collection("auth").Indexes().CreateMany(context.Background(), []mongo.IndexModel{
		{Keys: bsonx.Doc{{Key: "email"}}, Options: options.Index().SetUnique(true)},
	})
	if err != nil {
		log.Logger.Fatal("unable to create index", zap.Error(err))
	}

	return &authHandler{
		key: []byte("test-key"),
		c:   client.Database("comp").Collection("auth"),
	}
}
