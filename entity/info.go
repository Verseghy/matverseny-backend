package entity

import "time"

type Info struct {
	Time Time `bson:"time"`
}

type Time struct {
	StartDate time.Time `bson:"start_date"`
	EndDate   time.Time `bson:"end_date"`
}
