#include <stdio.h>
#include <stdint.h>
#include <errno.h>
#include <termios.h>
#include <fcntl.h>
#include <unistd.h>

/**
 * @brief Opens a serial device file
 * 
 * @param path The path to open
 * @param bauds The baud rate to configure
 * @return The device file descriptor or `-1` in case of an error
 */
int64_t serial_open(const uint8_t* path, uint64_t bauds) {
    // Open the device file nonblocking
    int devfile = open((const char*)path, O_RDWR | O_NONBLOCK);
    if (devfile < 0) {
        return -1;
    }

    // Make the file blocking again
    int flags = fcntl(devfile, F_GETFL, 0);
    if (fcntl(devfile, F_SETFL, flags & ~O_NONBLOCK) != 0) {
        return -1;
    }

    // Get the device attributes
    struct termios tty;
    if (tcgetattr(devfile, &tty) != 0) {
        return -1;
    }

    // Set the speed
    if (cfsetispeed(&tty, bauds) != 0) {
        return -1;
    }
    if (cfsetospeed(&tty, bauds) != 0) {
        return -1;
    }

    // Disable parity generation on output and parity checking for input
    tty.c_cflag &= ~PARENB;
    // Set one stop bit instead of two
    tty.c_cflag &= ~CSTOPB;
    // Use eight bit characters
    tty.c_cflag &= ~CSIZE;
    tty.c_cflag |= CS8;
    // Disable hardware flow control
    tty.c_cflag &= ~CRTSCTS;
    // Enable receiving
    tty.c_cflag |= CREAD;
    // Ignore modem control lines
    tty.c_cflag |= CLOCAL;
    // Disable canonical mode
    tty.c_lflag &= ~ICANON;
    // Disable INTR, QUIT, SUSP, or DSUSP signals
    tty.c_lflag &= ~ISIG;
    // Disable XON/XOFF
    tty.c_iflag &= ~(IXON | IXOFF);
    // Just allow the START character to restart output
    tty.c_iflag &= ~IXANY;
    // Disable special handling of various signals and parity-errors
    tty.c_iflag &= ~(IGNBRK | BRKINT | PARMRK | ISTRIP | INLCR | IGNCR | ICRNL);
    // Disable implementation-defined output processing
    tty.c_oflag &= ~OPOST;
    // Don't map NL to CR-NL on output
    tty.c_oflag &= ~ONLCR;
    // Minimum number of characters for noncanonical read
    tty.c_cc[VMIN] = 1;
    // Timeout in deciseconds for noncanonical read
    tty.c_cc[VTIME] = 0;
    
    // Apply the updated TTY settings
    if (tcsetattr(devfile, TCSANOW, &tty) != 0) {
        return -1;
    }
    return devfile;
}

/**
 * @brief Duplicates `fd`
 * 
 * @param fd The file descriptor to duplicate
 * @return The duplicate file descriptor or `-1` in case of an error
 */
int64_t serial_duplicate(int64_t fd) {
    return dup(fd);
}

/**
 * @brief Reads one byte from `fd`
 * 
 * @param fd The file descriptor to write to
 * @param buf The target buffer
 * @return `0` or `-1` on error
 */
int32_t serial_read_one(int64_t fd, uint8_t* buf) {
    // Try to read a single byte
    ssize_t read_ = read(fd, buf, 1);
    if (read_ == 0) {
        errno = EOF;
    }
    if (read_ < 1) {
        return -1;
    }
    return 0;
}

/**
 * @brief Writes one byte to `fd`
 * 
 * @param fd The file descriptor to write to
 * @param byte The byte to write
 * @return `0` or `-1` on error
 */
int32_t serial_write_one(int64_t fd, const uint8_t* byte) {
    // Write a single byte
    ssize_t written = write(fd, byte, 1);
    if (written == 0) {
        errno = EOF;
    }
    if (written < 1) {
        return -1;
    }

    // Flush output
    if (fsync(fd) != 0) {
        return -1;
    }
    return 0;
}

/**
 * @brief Closes `fd`
 * 
 * @param fd The file descriptor to close
 */
void serial_close(int64_t fd) {
    close(fd);
}
