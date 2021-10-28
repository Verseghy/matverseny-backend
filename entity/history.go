package entity

import (
	"go.mongodb.org/mongo-driver/bson/primitive"
	"time"
)

type History struct {
	ID        primitive.ObjectID `bson:"_id,omitempty"`
	Team      primitive.ObjectID `bson:"team"`
	ProblemID primitive.ObjectID `bson:"problem_id"`
	Time      time.Time          `bson:"time"`
	Value     int64              `bson:"value"`
}
