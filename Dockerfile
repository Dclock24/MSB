# Build stage
FROM golang:1.22-alpine AS build
WORKDIR /app
COPY trading_engine.go go.mod ./
RUN go build -o macro_strike_bot trading_engine.go

# Runtime
FROM alpine:3.20
WORKDIR /app
COPY --from=build /app/macro_strike_bot /app/
COPY market_analysis.jl /app/
COPY .env.example /app/
ENV LIVE_TRADING=0
ENTRYPOINT ["/app/macro_strike_bot"]
