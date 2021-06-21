package events

import (
	"bytes"
	"context"
	"encoding/gob"
	"github.com/google/uuid"
	"github.com/streadway/amqp"
	"go.uber.org/zap"
	"matverseny-backend/log"
	"time"
)

type TimeSubscriber struct {
	ID uuid.UUID
	Ch chan<- *TimeEvent
}

type TimeEvent struct {
	Start *time.Time
	End   *time.Time
}

func ConsumeTime(ctx context.Context) (<-chan *TimeEvent, error) {
	EnsureEvents()

	ch := make(chan *TimeEvent)

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
		TimeExchange,
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
				var p *TimeEvent
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

func PublishTime(event *TimeEvent) error {
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
	err = rch.Publish(TimeExchange, "", false, false, amqp.Publishing{
		ContentType: "application/protobuf",
		Body:        b.Bytes(),
	})
	if err != nil {
		return err
	}

	return nil
}
