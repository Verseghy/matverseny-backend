package log

import "go.uber.org/zap"

var Logger *zap.Logger

func EnsureLogger() {
	Logger, _ = zap.NewDevelopment()
	defer Logger.Sync()
}
