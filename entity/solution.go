package entity

import "go.mongodb.org/mongo-driver/bson/primitive"

type Solution struct {
	ID        primitive.ObjectID `bson:"_id,omitempty"`
	Team      primitive.ObjectID `bson:"team"`
	ProblemID primitive.ObjectID `bson:"problem_id"`
	Value     int64              `bson:"value"`
}
