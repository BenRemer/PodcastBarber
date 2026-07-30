# PodcastBarber

*Lets trim a little off the top*

PodcastBarber is a service to help trim ads out of the podcasts you want to listen to.

## Getting Started

The project is packaged up to easily run using Docker Compose. There are env vars that can be set but all have good defaults.

### Installation & Usage

1. Clone the repository:

   ```bash
   git clone https://github.com/BenRemer/PodcastBarber.git
   cd PodcastBarber
   ```

2. Review the `docker-compose.yml` file to adjust any volume mounts or environment variables for your specific setup.

#### Option A: Standard Docker Compose

Spin up the container in the background:

```bash
docker compose up -d
```

To pull the container down later:

```bash
docker compose down
```

#### Option B: Using `buildandfollow`

If you want to build the image from scratch and immediately tail the output logs you can use the included script:

```bash
chmod +x buildandfollow.sh  # If it needs execute permissions
./buildandfollow.sh
```

## Contributing

Feel free to open an issue or submit a pull request if you want to help improve things.

## License

MIT
