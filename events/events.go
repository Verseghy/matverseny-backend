package events

import (
	"os"
	"time"

	"github.com/streadway/amqp"
	"matverseny-backend/log"
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
		log.Logger.Info("Trying to connect to rabbitmq...")
		s := envOrDefaultString("RABBITMQ_CONNSTRING", "amqp://user:bitnami@rabbitmq:5672/")

		var conn *amqp.Connection
		t := time.Second
		for i := 0; i < 6; i++ {
			var err error
			conn, err = amqp.Dial(s)
			if err != nil {
				if i == 5 {
					panic(err)
				}
				time.Sleep(t)
				t *= 2

				continue
			}

			break
		}
		log.Logger.Info("Connected to rabbitmq")

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
			"topic",
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
