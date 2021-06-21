package events

import (
	"github.com/streadway/amqp"
	"os"
)

const (
	ProblemsExchange  = "problems"
	SolutionsExchange = "solutions"
	TimeExchange      = "time"
)

type Events struct {
	Conn *amqp.Connection
}

var e *Events

func envOrDefaultString(env, def string) string {
	if val, ok := os.LookupEnv(env); ok {
		return val
	}

	return def
}

func EnsureEvents() {
	if e == nil {
		s := envOrDefaultString("RABBITMQ_CONNSTRING", "amqp://user:bitnami@rabbitmq:5672/")
		conn, err := amqp.Dial(s)
		if err != nil {
			panic(err)
		}

		ch, err := conn.Channel()
		if err != nil {
			panic(err)
		}
		defer ch.Close()

		err = ch.ExchangeDeclare(
			ProblemsExchange,
			"fanout",
			true,
			false,
			false,
			false,
			nil,
		)
		if err != nil {
			panic(err)
		}

		err = ch.ExchangeDeclare(
			SolutionsExchange,
			"fanout",
			true,
			false,
			false,
			false,
			nil,
		)
		if err != nil {
			panic(err)
		}

		err = ch.ExchangeDeclare(
			TimeExchange,
			"fanout",
			true,
			false,
			false,
			false,
			nil,
		)
		if err != nil {
			panic(err)
		}

		e = &Events{
			Conn: conn,
		}
	}
}
