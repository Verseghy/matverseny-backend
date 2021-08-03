package jwt

import (
	"context"
	"errors"
	"github.com/dgrijalva/jwt-go"
	grpcauth "github.com/grpc-ecosystem/go-grpc-middleware/auth"
	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/mongo"
	"go.uber.org/zap"
	"google.golang.org/grpc/metadata"
	"matverseny-backend/entity"
	"matverseny-backend/errs"
	"matverseny-backend/log"
	"strings"
	"time"
)

var (
	ErrExpired = errors.New("token expired")
)

type SuperAdminClaims struct {
	IsSA bool `json:"is_sa"`
	*jwt.StandardClaims
}

type RefreshClaims struct {
	UserID  string `json:"user_id"`
	IsAdmin bool   `json:"is_admin"`
	*jwt.StandardClaims
}

type ctxAccessClaims struct{}

type AccessClaims struct {
	UserID  string `json:"user_id"`
	IsAdmin bool   `json:"is_admin"`
	Team    string `json:"team"`
	*jwt.StandardClaims
}

type JWT interface {
	NewRefreshToken(ctx context.Context, user *entity.User) (string, error)
	NewAccessToken(ctx context.Context, user *entity.User) (string, error)
	ValidateRefreshToken(token string) (*RefreshClaims, error)
	ValidateSuperAdminToken() grpcauth.AuthFunc
	ValidateAccessToken() grpcauth.AuthFunc
}

type jwtImpl struct {
	key    []byte
	cTeams *mongo.Collection
}

func (j *jwtImpl) NewRefreshToken(ctx context.Context, user *entity.User) (string, error) {
	token := jwt.NewWithClaims(jwt.SigningMethodHS512, &RefreshClaims{
		UserID:  user.ID.Hex(),
		IsAdmin: user.IsAdmin,
		StandardClaims: &jwt.StandardClaims{
			ExpiresAt: time.Now().Add(time.Hour * 24 * 30 * 6).Unix(),
			IssuedAt:  time.Now().Unix(),
			Issuer:    "verseghy-matverseny",
		},
	})

	ss, err := token.SignedString(j.key)
	if err != nil {
		log.Logger.Error("signing failure", zap.Error(err))
		return "", err
	}

	return ss, nil
}

func (j *jwtImpl) NewAccessToken(ctx context.Context, user *entity.User) (string, error) {
	team := &entity.Team{}
	err := j.cTeams.FindOne(ctx, bson.M{"members": user.ID}).Decode(team)
	if err != nil && err != mongo.ErrNoDocuments {
		log.Logger.Error("database error", zap.Error(err))
		return "", errs.ErrDatabase
	}

	token := jwt.NewWithClaims(jwt.SigningMethodHS512, &AccessClaims{
		UserID: user.ID.Hex(),
		Team: func() string {
			if team.ID.IsZero() {
				return ""
			}

			return team.ID.Hex()
		}(),
		IsAdmin: user.IsAdmin,
		StandardClaims: &jwt.StandardClaims{
			ExpiresAt: time.Now().Add(time.Hour * 24).Unix(),
			IssuedAt:  time.Now().Unix(),
			Issuer:    "verseghy-matverseny",
		},
	})

	ss, err := token.SignedString(j.key)
	if err != nil {
		log.Logger.Error("signing failure", zap.Error(err))
		return "", err
	}

	return ss, nil
}

func (j *jwtImpl) ValidateRefreshToken(token string) (*RefreshClaims, error) {
	t, err := jwt.ParseWithClaims(token, &RefreshClaims{}, func(token *jwt.Token) (interface{}, error) {
		return j.key, nil
	})
	if err != nil {
		log.Logger.Debug("parse failure", zap.Error(err))
		return nil, err
	}

	c := t.Claims.(*RefreshClaims)
	if c.ExpiresAt < time.Now().Unix() {
		return nil, ErrExpired
	}

	return c, nil
}

func GetClaimsFromCtx(ctx context.Context) (*AccessClaims, bool) {
	val, ok := ctx.Value(ctxAccessClaims{}).(*AccessClaims)
	return val, ok
}

func (j *jwtImpl) ValidateSuperAdminToken() grpcauth.AuthFunc {
	return func(ctx context.Context) (context.Context, error) {
		md, ok := metadata.FromIncomingContext(ctx)
		if !ok {
			return ctx, errs.ErrUnauthorized
		}

		s := md.Get("Authorization")
		if len(s) != 1 {
			return nil, errs.ErrUnauthorized
		}
		token := strings.TrimPrefix(s[0], "Bearer: ")

		t, err := jwt.ParseWithClaims(token, &SuperAdminClaims{}, func(token *jwt.Token) (interface{}, error) {
			return j.key, nil
		})
		if err != nil {
			log.Logger.Debug("parse failure", zap.Error(err))
			return nil, err
		}

		c := t.Claims.(*SuperAdminClaims)
		if c.ExpiresAt < time.Now().Unix() {
			return nil, ErrExpired
		}

		if !c.IsSA {
			return nil, errs.ErrUnauthorized
		}

		return ctx, nil
	}
}

func (j *jwtImpl) ValidateAccessToken() grpcauth.AuthFunc {
	return func(ctx context.Context) (context.Context, error) {
		md, ok := metadata.FromIncomingContext(ctx)
		if !ok {
			return ctx, errs.ErrUnauthorized
		}

		s := md.Get("Authorization")
		if len(s) != 1 {
			return nil, errs.ErrUnauthorized
		}
		token := strings.TrimPrefix(s[0], "Bearer: ")

		t, err := jwt.ParseWithClaims(token, &AccessClaims{}, func(token *jwt.Token) (interface{}, error) {
			return j.key, nil
		})
		if err != nil {
			log.Logger.Debug("parse failure", zap.Error(err))
			return nil, errs.ErrJWT
		}

		c := t.Claims.(*AccessClaims)
		if c.ExpiresAt < time.Now().Unix() {
			return nil, ErrExpired
		}

		ctx = context.WithValue(ctx, ctxAccessClaims{}, c)

		return ctx, nil
	}
}

func NewJWT(client *mongo.Client, key []byte) *jwtImpl {
	return &jwtImpl{
		key:    key,
		cTeams: client.Database("comp").Collection("teams"),
	}
}
