package jwt

import (
	"context"
	"errors"
	"github.com/dgrijalva/jwt-go"
	grpc_auth "github.com/grpc-ecosystem/go-grpc-middleware/auth"
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

type RefreshClaims struct {
	UserID  string `json:"user_id"`
	IsAdmin bool   `json:"is_admin"`
	*jwt.StandardClaims
}

type ctxAccessClaims struct{}

type AccessClaims struct {
	UserID string `json:"user_id"`
	Team   string `json:"team"`
	*jwt.StandardClaims
}

func NewRefreshToken(user *entity.User, key []byte) (string, error) {
	token := jwt.NewWithClaims(jwt.SigningMethodHS512, &RefreshClaims{
		UserID:  user.ID.Hex(),
		IsAdmin: user.IsAdmin,
		StandardClaims: &jwt.StandardClaims{
			ExpiresAt: time.Now().Add(time.Hour * 24 * 30 * 6).Unix(),
			IssuedAt:  time.Now().Unix(),
			Issuer:    "verseghy-matverseny",
		},
	})

	ss, err := token.SignedString(key)
	if err != nil {
		log.Logger.Error("signing failure", zap.Error(err))
		return "", err
	}

	return ss, nil
}

func NewAccessToken(user *entity.User, key []byte) (string, error) {
	token := jwt.NewWithClaims(jwt.SigningMethodHS512, &AccessClaims{
		UserID: user.ID.Hex(),
		Team:   user.Team,
		StandardClaims: &jwt.StandardClaims{
			ExpiresAt: time.Now().Add(time.Hour * 24).Unix(),
			IssuedAt:  time.Now().Unix(),
			Issuer:    "verseghy-matverseny",
		},
	})

	ss, err := token.SignedString(key)
	if err != nil {
		log.Logger.Error("signing failure", zap.Error(err))
		return "", err
	}

	return ss, nil
}

func ValidateRefreshToken(token string, key []byte) (*RefreshClaims, error) {
	t, err := jwt.ParseWithClaims(token, &RefreshClaims{}, func(token *jwt.Token) (interface{}, error) {
		return key, nil
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

func ValidateAccessToken(key []byte) grpc_auth.AuthFunc {
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
			return key, nil
		})
		if err != nil {
			log.Logger.Debug("parse failure", zap.Error(err))
			return nil, err
		}

		c := t.Claims.(*AccessClaims)
		if c.ExpiresAt < time.Now().Unix() {
			return nil, ErrExpired
		}

		ctx = context.WithValue(ctx, ctxAccessClaims{}, c)

		return ctx, nil
	}
}
