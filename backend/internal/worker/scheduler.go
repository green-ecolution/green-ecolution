package worker

import (
	"context"
	"time"

	"github.com/green-ecolution/backend/internal/logger"
)

type Scheduler struct {
	interval time.Duration
	work     SchedulerWork
}

func NewScheduler(interval time.Duration, w SchedulerWork) *Scheduler {
	return &Scheduler{
		interval: interval,
		work:     w,
	}
}

type SchedulerWork interface {
	Do(context.Context) error
}

type SchedulerFunc func(context.Context) error

func (s SchedulerFunc) Do(ctx context.Context) error {
	return s(ctx)
}

func (s *Scheduler) Run(ctx context.Context) {
	log := logger.GetLogger(ctx)
	ticker := time.NewTicker(s.interval)
	defer ticker.Stop()

	if err := s.work.Do(ctx); err != nil {
		log.Error("error during initial process execution", "error", err)
	}

	for {
		select {
		case <-ticker.C:
			if err := s.work.Do(ctx); err != nil {
				log.Error("error during process execution", "error", err)
			}
		case <-ctx.Done():
			log.Debug("stopping scheduler")
			return
		}
	}
}
