package entity

import (
	"go.mongodb.org/mongo-driver/bson/primitive"
	"time"
)

type PasswordReset struct {
	ID     primitive.ObjectID `bson:"_id"`
	UserID primitive.ObjectID `bson:"user_id"`
	Token  string             `bson:"token"`
	TTL    time.Time          `bson:"ttl"`
}
