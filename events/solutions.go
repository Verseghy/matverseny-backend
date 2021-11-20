package events

import (
	"bytes"
	"context"
	"encoding/gob"

	"github.com/google/uuid"
	"github.com/streadway/amqp"
	"go.mongodb.org/mongo-driver/bson/primitive"
	"go.uber.org/zap"
	"matverseny-backend/log"
)

type SolutionSubscriber struct {
	ID   uuid.UUID
	Team string
	Ch   chan<- *SolutionEvent
}

type SolutionType uint32

const (
	SChange SolutionType = iota
	SDelete
)

type SolutionEvent struct {
	Type      SolutionType
	ProblemID primitive.ObjectID
	Team      primitive.ObjectID
	Value     int64
}

func ConsumeAdminSolution(ctx context.Context) (<-chan *SolutionEvent, error) {
	EnsureEvents()

	ch := make(chan *SolutionEvent)

	rch, err := e.Conn.Channel()
	if err != nil {
		panic(err)
	}
	q, err := rch.QueueDeclare("", false, true, true, false, nil)
	if err != nil {
		return nil, err
	}

	err = rch.QueueBind(
		q.Name,
		"#", // matches all keys
		SolutionsExchange,
		false,
		nil,
	)
	if err != nil {
		return nil, err
	}

	msgs, err := rch.Consume(q.Name, "", true, false, false, false, nil)
	if err != nil {
		return nil, err
	}

	go func() {
		c := make(chan amqp.Delivery)

		go func() {
			for d := range msgs {
				c <- d
			}
		}()

		for {
			select {
			case <-ctx.Done():
				err := rch.Close()
				if err != nil {
					log.Logger.Error("unable to close channel", zap.Error(err))
				}
				return
			case d := <-c:
				var p *SolutionEvent
				b := bytes.NewReader(d.Body)
				err := gob.NewDecoder(b).Decode(&p)
				if err != nil {
					log.Logger.Error("unable to decode event", zap.Error(err))
				}

				ch <- p
			}
		}
	}()

	return ch, nil
}

func ConsumeSolution(ctx context.Context, team string) (<-chan *SolutionEvent, error) {
	EnsureEvents()

	ch := make(chan *SolutionEvent)

	rch, err := e.Conn.Channel()
	if err != nil {
		panic(err)
	}
	q, err := rch.QueueDeclare("", false, true, true, false, nil)
	if err != nil {
		return nil, err
	}

	err = rch.QueueBind(
		q.Name,
		team,
		SolutionsExchange,
		false,
		nil,
	)
	if err != nil {
		return nil, err
	}

	msgs, err := rch.Consume(q.Name, "", true, false, false, false, nil)
	if err != nil {
		return nil, err
	}

	go func() {
		c := make(chan amqp.Delivery)

		go func() {
			for d := range msgs {
				c <- d
			}
		}()

		for {
			select {
			case <-ctx.Done():
				err := rch.Close()
				if err != nil {
					log.Logger.Error("unable to close channel", zap.Error(err))
				}
				return
			case d := <-c:
				var p *SolutionEvent
				b := bytes.NewReader(d.Body)
				err := gob.NewDecoder(b).Decode(&p)
				if err != nil {
					log.Logger.Error("unable to decode event", zap.Error(err))
				}

				ch <- p
			}
		}
	}()

	return ch, nil
}

func PublishSolution(event *SolutionEvent) error {
	EnsureEvents()

	var b bytes.Buffer
	err := gob.NewEncoder(&b).Encode(event)
	if err != nil {
		return err
	}

	rch, err := e.Conn.Channel()
	if err != nil {
		panic(err)
	}
	defer rch.Close()
	err = rch.Publish(SolutionsExchange, event.Team.Hex(), false, false, amqp.Publishing{
		ContentType: "application/protobuf",
		Body:        b.Bytes(),
	})
	if err != nil {
		return err
	}

	return nil
}
