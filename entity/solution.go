package entity

import "go.mongodb.org/mongo-driver/bson/primitive"

type Solution struct {
	ID        primitive.ObjectID `bson:"_id"`
	Team      primitive.ObjectID `bson:"team"`
	ProblemID primitive.ObjectID `bson:"problem_id"`
	Value     int64              `bson:"value"`
}
