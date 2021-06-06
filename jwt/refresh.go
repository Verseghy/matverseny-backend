package jwt

import (
	"errors"
	"github.com/dgrijalva/jwt-go"
	"go.uber.org/zap"
	"matverseny-backend/entity"
	"matverseny-backend/log"
	"time"
)

var (
	ErrExpired = errors.New("token expired")
)

type RefreshClaims struct {
	UserID string `json:"user_id"`
	*jwt.StandardClaims
}

type AccessClaims struct {
	UserID  string `json:"user_id"`
	IsAdmin bool   `json:"is_admin"`
	Team    string `json:"team"`
	*jwt.StandardClaims
}

func NewRefreshToken(user *entity.User, key []byte) (string, error) {
	token := jwt.NewWithClaims(jwt.SigningMethodHS512, &RefreshClaims{
		UserID: user.ID.Hex(),
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
		UserID:  user.ID.Hex(),
		IsAdmin: user.IsAdmin,
		Team:    user.Team,
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

func ValidateAccessToken(token string, key []byte) (*AccessClaims, error) {
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

	return c, nil
}
