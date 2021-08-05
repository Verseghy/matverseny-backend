package int

import (
	"context"
	"github.com/golang-jwt/jwt/v4"
	. "github.com/onsi/ginkgo"
	. "github.com/onsi/gomega"
	"google.golang.org/grpc"
	"matverseny-backend/errs"
	jwt2 "matverseny-backend/jwt"
	pb "matverseny-backend/proto"
	"time"
)

var key = []byte("test-key")

var _ = Describe("Auth", func() {
	var opts []grpc.DialOption
	opts = append(opts, grpc.WithInsecure())

	conn, err := grpc.Dial("localhost:6969", opts...)
	Expect(err).To(BeNil())

	authClient := pb.NewAuthClient(conn)

	BeforeEach(func() {
		cleanupMongo()
	})

	Describe("Register", func() {
		Specify("happy path", func() {
			res, err := authClient.Register(context.Background(), &pb.RegisterRequest{
				Email:    "test@test.test",
				Password: "testtest",
				Name:     "test",
				School:   "test",
				Class:    0,
			})
			Expect(err).To(BeNil())

			Expect(res.RefreshToken).NotTo(BeNil())
			Expect(res.AccessToken).NotTo(BeNil())

			rt, err := jwt.ParseWithClaims(res.RefreshToken, &jwt2.RefreshClaims{}, func(token *jwt.Token) (interface{}, error) {
				return key, nil
			})
			Expect(err).To(BeNil())
			c, ok := rt.Claims.(*jwt2.RefreshClaims)
			Expect(ok).To(BeTrue())
			Expect(c.ExpiresAt).To(Satisfy(func(t int64) bool { return time.Now().Unix() < t }))
			Expect(c.UserID).NotTo(BeEmpty())
			Expect(c.IsAdmin).To(BeFalse())

			at, err := jwt.ParseWithClaims(res.AccessToken, &jwt2.AccessClaims{}, func(token *jwt.Token) (interface{}, error) {
				return key, nil
			})
			Expect(err).To(BeNil())
			c2, ok := at.Claims.(*jwt2.AccessClaims)
			Expect(ok).To(BeTrue())
			Expect(c2.ExpiresAt).To(Satisfy(func(t int64) bool { return time.Now().Unix() < t }))
			Expect(c2.UserID).NotTo(BeEmpty())
			Expect(c2.IsAdmin).To(BeFalse())
			Expect(c2.Team).To(BeEmpty())
		})
		Specify("sad path - name empty", func() {
			_, err := authClient.Register(context.Background(), &pb.RegisterRequest{
				Email:    "test@test.test",
				Password: "testtest",
				Name:     "",
				School:   "test",
				Class:    0,
			})
			Expect(err).NotTo(BeNil())
			Expect(err.Error()).To(ContainSubstring(errs.ErrNameRequired.Error()))
		})
		Specify("sad path - wrong email", func() {
			_, err := authClient.Register(context.Background(), &pb.RegisterRequest{
				Email:    "test-test.test.test",
				Password: "testtest",
				Name:     "test",
				School:   "test",
				Class:    0,
			})
			Expect(err).NotTo(BeNil())
			Expect(err.Error()).To(ContainSubstring(errs.ErrEmailAddressFormat.Error()))
		})
		Specify("sad path - password empty", func() {
			_, err := authClient.Register(context.Background(), &pb.RegisterRequest{
				Email:    "test@test.test",
				Password: "",
				Name:     "test",
				School:   "test",
				Class:    0,
			})
			Expect(err).NotTo(BeNil())
			Expect(err.Error()).To(ContainSubstring(errs.ErrPasswordRequired.Error()))
		})
		Specify("sad path - school empty", func() {
			_, err := authClient.Register(context.Background(), &pb.RegisterRequest{
				Email:    "test@test.test",
				Password: "testtest",
				Name:     "test",
				School:   "",
				Class:    0,
			})
			Expect(err).NotTo(BeNil())
			Expect(err.Error()).To(ContainSubstring(errs.ErrSchoolRequired.Error()))
		})
		Specify("sad path - already has account", func() {
			_, err := authClient.Register(context.Background(), &pb.RegisterRequest{
				Email:    "test@test.test",
				Password: "testtest",
				Name:     "test",
				School:   "test",
				Class:    0,
			})
			Expect(err).To(BeNil())
			_, err = authClient.Register(context.Background(), &pb.RegisterRequest{
				Email:    "test@test.test",
				Password: "testtest",
				Name:     "test",
				School:   "test",
				Class:    0,
			})
			Expect(err).NotTo(BeNil())
			Expect(err.Error()).To(ContainSubstring(errs.ErrAlreadyExists.Error()))
		})
	})

	Describe("Login", func() {
		email := "test@test.test"
		password := "testtest"

		BeforeEach(func() {
			res, err := authClient.Register(context.Background(), &pb.RegisterRequest{
				Email:    email,
				Password: password,
				Name:     "test",
				School:   "test",
				Class:    0,
			})
			Expect(err).To(BeNil())
			Expect(res.RefreshToken).NotTo(BeEmpty())
			Expect(res.AccessToken).NotTo(BeEmpty())
		})

		Specify("happy path", func() {
			res, err := authClient.Login(context.Background(), &pb.LoginRequest{
				Email:    email,
				Password: password,
			})
			Expect(err).To(BeNil())

			Expect(res.RefreshToken).NotTo(BeNil())
			Expect(res.AccessToken).NotTo(BeNil())

			rt, err := jwt.ParseWithClaims(res.RefreshToken, &jwt2.RefreshClaims{}, func(token *jwt.Token) (interface{}, error) {
				return key, nil
			})
			Expect(err).To(BeNil())
			c, ok := rt.Claims.(*jwt2.RefreshClaims)
			Expect(ok).To(BeTrue())
			Expect(c.ExpiresAt).To(Satisfy(func(t int64) bool { return time.Now().Unix() < t }))
			Expect(c.UserID).NotTo(BeEmpty())
			Expect(c.IsAdmin).To(BeFalse())

			at, err := jwt.ParseWithClaims(res.AccessToken, &jwt2.AccessClaims{}, func(token *jwt.Token) (interface{}, error) {
				return key, nil
			})
			Expect(err).To(BeNil())
			c2, ok := at.Claims.(*jwt2.AccessClaims)
			Expect(ok).To(BeTrue())
			Expect(c2.ExpiresAt).To(Satisfy(func(t int64) bool { return time.Now().Unix() < t }))
			Expect(c2.UserID).NotTo(BeEmpty())
			Expect(c2.IsAdmin).To(BeFalse())
			Expect(c2.Team).To(BeEmpty())
		})
		Specify("sad path - wrong password", func() {
			_, err := authClient.Login(context.Background(), &pb.LoginRequest{
				Email:    email,
				Password: password + " - wrong",
			})
			Expect(err).NotTo(BeNil())
			Expect(err.Error()).To(ContainSubstring(errs.ErrInvalidEmailOrPassword.Error()))
		})
	})
})
