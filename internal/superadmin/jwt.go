package superadmin

import (
	"fmt"
	"github.com/golang-jwt/jwt/v4"
	jwt2 "matverseny-backend/jwt"
	"time"
)

func GenerateToken(exp time.Time, key string) (string, error) {
	token := jwt.NewWithClaims(jwt.SigningMethodHS512, &jwt2.SuperAdminClaims{
		IsSA: true,
		StandardClaims: &jwt.StandardClaims{
			ExpiresAt: exp.Unix(),
			IssuedAt:  time.Now().Unix(),
			Issuer:    "verseghy-matverseny",
		},
	})

	ss, err := token.SignedString([]byte(key))
	if err != nil {
		fmt.Println("Signing failure:", err)
		return "", err
	}

	return ss, nil
}
