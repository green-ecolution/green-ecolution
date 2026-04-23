package entities

type WateringStatus string

const (
	WateringStatusGood        WateringStatus = "good"
	WateringStatusModerate    WateringStatus = "moderate"
	WateringStatusBad         WateringStatus = "bad"
	WateringStatusJustWatered WateringStatus = "just watered"
	WateringStatusUnknown     WateringStatus = "unknown"
)
