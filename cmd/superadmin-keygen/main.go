package main

import (
	"flag"
	"fmt"
	"github.com/golang-jwt/jwt/v4"
	jwt2 "matverseny-backend/jwt"
	"os"
	"time"
)

func main() {
	s := flag.String("key", "", "Encryption key used to sign SA JWT")
	e := flag.String("exp", time.Now().Add(time.Hour*24*365/2).Format(time.RFC3339), "RFC3339 time of the expiration date")
	flag.Parse()

	if s == nil || *s == "" {
		fmt.Println("--key is required")
		os.Exit(1)
	}

	exp, err := time.Parse(time.RFC3339, *e)
	if err != nil {
		fmt.Println("--exp invalid time")
		os.Exit(1)
	}

	token := jwt.NewWithClaims(jwt.SigningMethodHS512, &jwt2.SuperAdminClaims{
		IsSA: true,
		StandardClaims: &jwt.StandardClaims{
			ExpiresAt: exp.Unix(),
			IssuedAt:  time.Now().Unix(),
			Issuer:    "verseghy-matverseny",
		},
	})

	ss, err := token.SignedString([]byte(*s))
	if err != nil {
		fmt.Println("Signing failure:", err)
		os.Exit(1)
	}

	fmt.Println("Token successfully generated:", ss)
}
