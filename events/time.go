package events

import (
	"context"
	"github.com/google/uuid"
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

func ConsumeTime(ctx context.Context) <-chan *TimeEvent {
	ensureEvents()

	ch := make(chan *TimeEvent)
	e.lock.Lock()
	ID, err := uuid.NewUUID()
	if err != nil {
		panic(err)
	}
	e.timeSubscribers = append(e.timeSubscribers, &TimeSubscriber{ID: ID, Ch: ch})
	go func() {
		<-ctx.Done()
		e.lock.Lock()
		defer e.lock.Unlock()

		for k, v := range e.timeSubscribers {
			if v.ID == ID {
				a := e.timeSubscribers
				a[k] = a[len(a)-1]
				a[len(a)-1] = nil
				a = a[:len(a)-1]
				break
			}
		}
	}()
	e.lock.Unlock()

	return ch
}

func PublishTime(event *TimeEvent) {
	ensureEvents()

	e.lock.Lock()
	defer e.lock.Unlock()

	for _, v := range e.timeSubscribers {
		v.Ch <- event
	}
}
