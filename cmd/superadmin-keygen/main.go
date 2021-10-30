package main

import (
	"flag"
	"fmt"
	"matverseny-backend/internal/superadmin"
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

	ss, err := superadmin.GenerateToken(exp, *s)
	if err != nil {
		os.Exit(1)
	}

	fmt.Println("Token successfully generated:", ss)
}
