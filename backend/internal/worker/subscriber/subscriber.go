package subscriber

import (
	"context"

	"github.com/green-ecolution/green-ecolution/backend/internal/application/ports"
	entities "github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
)

type UpdateTreeSubscriber struct {
	tcs ports.TreeClusterService
}

func NewUpdateTreeSubscriber(tcs ports.TreeClusterService) *UpdateTreeSubscriber {
	return &UpdateTreeSubscriber{
		tcs: tcs,
	}
}

func (s *UpdateTreeSubscriber) EventType() entities.EventType {
	return entities.EventTypeUpdateTree
}

func (s *UpdateTreeSubscriber) HandleEvent(ctx context.Context, e entities.Event) error {
	event := e.(entities.EventUpdateTree)
	return s.tcs.HandleUpdateTree(ctx, &event)
}

type CreateTreeSubscriber struct {
	tcs ports.TreeClusterService
}

func NewCreateTreeSubscriber(tcs ports.TreeClusterService) *CreateTreeSubscriber {
	return &CreateTreeSubscriber{
		tcs: tcs,
	}
}

func (s *CreateTreeSubscriber) EventType() entities.EventType {
	return entities.EventTypeCreateTree
}

func (s *CreateTreeSubscriber) HandleEvent(ctx context.Context, e entities.Event) error {
	event := e.(entities.EventCreateTree)
	return s.tcs.HandleCreateTree(ctx, &event)
}

type DeleteTreeSubscriber struct {
	tcs ports.TreeClusterService
}

func NewDeleteTreeSubscriber(tcs ports.TreeClusterService) *DeleteTreeSubscriber {
	return &DeleteTreeSubscriber{
		tcs: tcs,
	}
}

func (s *DeleteTreeSubscriber) EventType() entities.EventType {
	return entities.EventTypeDeleteTree
}

func (s *DeleteTreeSubscriber) HandleEvent(ctx context.Context, e entities.Event) error {
	event := e.(entities.EventDeleteTree)
	return s.tcs.HandleDeleteTree(ctx, &event)
}

type CreateSensorDataSubscriber struct {
	tcSvc   ports.TreeClusterService
	treeSvc ports.TreeService
}

func NewSensorDataSubscriber(tcSvc ports.TreeClusterService, treeSvc ports.TreeService) *CreateSensorDataSubscriber {
	return &CreateSensorDataSubscriber{
		tcSvc:   tcSvc,
		treeSvc: treeSvc,
	}
}

func (s *CreateSensorDataSubscriber) EventType() entities.EventType {
	return entities.EventTypeNewSensorData
}

func (s *CreateSensorDataSubscriber) HandleEvent(ctx context.Context, e entities.Event) error {
	event := e.(entities.EventNewSensorData)
	if err := s.treeSvc.HandleNewSensorData(ctx, &event); err != nil {
		return err
	}

	return s.tcSvc.HandleNewSensorData(ctx, &event)
}

type UpdateWateringPlanSubscriber struct {
	tcSvc ports.TreeClusterService
}

func NewUpdateWateringPlanSubscriber(tcSvc ports.TreeClusterService) *UpdateWateringPlanSubscriber {
	return &UpdateWateringPlanSubscriber{
		tcSvc: tcSvc,
	}
}

func (s *UpdateWateringPlanSubscriber) EventType() entities.EventType {
	return entities.EventTypeUpdateWateringPlan
}

func (s *UpdateWateringPlanSubscriber) HandleEvent(ctx context.Context, e entities.Event) error {
	event := e.(entities.EventUpdateWateringPlan)
	if err := s.tcSvc.HandleUpdateWateringPlan(ctx, &event); err != nil {
		return err
	}

	return s.tcSvc.HandleUpdateWateringPlan(ctx, &event)
}
