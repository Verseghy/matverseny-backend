package events

import (
	"bytes"
	"context"
	"encoding/gob"
	"github.com/google/uuid"
	"github.com/streadway/amqp"
	"go.uber.org/zap"
	"matverseny-backend/entity"
	"matverseny-backend/log"
)

type ProblemSubscriber struct {
	ID uuid.UUID
	Ch chan<- *ProblemEvent
}

type ProblemType uint32

const (
	PChange ProblemType = iota
	PDelete
	PSwap
	PCreate
)

type ProblemEvent struct {
	Type    ProblemType
	Problem *entity.Problem
	A       *entity.Problem
	B       *entity.Problem
	At      uint32
}

func ConsumeProblem(ctx context.Context) (<-chan *ProblemEvent, error) {
	EnsureEvents()

	ch := make(chan *ProblemEvent)

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
		"",
		ProblemsExchange,
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
				var p *ProblemEvent
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

func PublishProblem(event *ProblemEvent) error {
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
	err = rch.Publish(ProblemsExchange, "", false, false, amqp.Publishing{
		ContentType: "application/protobuf",
		Body:        b.Bytes(),
	})
	if err != nil {
		return err
	}

	return nil
}
