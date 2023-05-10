# Use the latest Rust image as the base image
FROM rust:latest

# Install required system packages and Python 3 dependencies
RUN apt-get update && apt-get install -y python3 python3-pip libasound2-dev
#RUN pip3 install -r tests/requirements.txt

# Set the working directory to /app
WORKDIR /app

# Copy the application files to the container
COPY . /app

# Build the Rust application
RUN cargo build --release

EXPOSE 5000
EXPOSE 5001

# Run the application
CMD ["bash"]
