FROM oven/bun AS build # tag it as build

WORKDIR /app

COPY bun.lockb .
COPY package.json .

RUN bun install --frozen-lockfile

COPY src ./src

# compile everything to a binary called cli which includes the bun runtime
RUN bun build ./src/index.ts --compile --outfile cli

FROM ubuntu:22.04 # use a smaller image without bun

WORKDIR /app

# copy the compiled binary from the build image
COPY --from=build /app/cli /app/cli

# execute the binary!
CMD ["/app/cli"]
