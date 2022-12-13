#include <stdio.h>
#include <stdint.h>
#include <errno.h>
#include <termios.h>
#include <fcntl.h>
#include <unistd.h>

/**
 * @brief Opens a serial device file
 * 
 * @param fd The target pointer to store the file descriptor
 * @param path The path to open
 * @param bauds The baud rate to configure
 * @return `NULL` on success or a static error string in case of a failure
 */
const char* serial_open(int64_t* fd, const uint8_t* path, uint64_t bauds) {
    // Open the device file nonblocking
    *fd = open((const char*)path, O_RDWR | O_NONBLOCK);
    if (*fd < 0) {
        return "failed to open serial file";
    }

    // Make the file blocking again
    int flags = fcntl(*fd, F_GETFL, 0);
    if (fcntl(*fd, F_SETFL, flags & ~O_NONBLOCK) != 0) {
        return "failed to set mode to blocking";
    }

    // Get the device attributes
    struct termios tty;
    if (tcgetattr(*fd, &tty) != 0) {
        return "failed to get device attributes";
    }

    // Set the speed
    if (cfsetspeed(&tty, bauds) != 0) {
        return "failed to set baudrate";
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
    if (tcsetattr(*fd, TCSANOW, &tty) != 0) {
        return "failed to apply TTY settings";
    }
    return NULL;
}

/**
 * @brief Duplicates a file descriptor
 * 
 * @param fd The target pointer to store the file descriptor
 * @param org The file descriptor to duplicate
 * @return The duplicate file descriptor or `-1` in case of an error
 */
const char* serial_duplicate(int64_t* fd, int64_t org) {
    *fd = (int64_t)dup((int)org);
    if (*fd < 0) {
        return "failed to duplicate file descriptor";
    }
    return NULL;
}

/**
 * @brief Reads some data into `buf` and updates `pos` accordingly
 * 
 * @note This function attempts to always read at least one byte
 * 
 * @param buf The target buffer to read into
 * @param pos The position within the buffer
 * @param capacity The total capacity of the buffer
 * @param fd The file descriptor to read from
 * @return `NULL` on success or a static error string in case of a failure
 */
const char* serial_read_buf(uint8_t* buf, size_t* pos, size_t capacity, int64_t fd) {
    // Return if the buffer is exhausted
    const size_t available = capacity - *pos;
    if (available == 0) {
        return NULL;
    }

    retry: {
        // Read some data
        ssize_t read_ = read((int)fd, buf + *pos, available);
        if (read_ < 0 && errno == EINTR) {
            goto retry;
        }
        
        // Parse the result
        if (read_ == 0) {
            errno = EOF;
            return "failed to read some data due to EOF";
        }
        if (read_ < 0) {
            return "failed to read some data";
        }

        // Update the buffer
        *pos += read_;
        return NULL;
    }
}

/**
 * @brief Writes some data from `buf` and updates `pos` accordingly
 * 
 * @param fd The file descriptor to write to
 * @param buf The buffer to write to
 * @param pos The position within the buffer
 * @param capacity The total capacity of the buffer
 * @return `NULL` on success or a static error string in case of a failure
 */
const char* serial_write_buf(int64_t fd, const uint8_t* buf, size_t* pos, size_t capacity) {
    // Return if the buffer is exhausted
    const size_t available = capacity - *pos;
    if (available == 0) {
        return NULL;
    }

    retry: {
        // Write some data
        ssize_t written = write((int)fd, buf + *pos, available);
        if (written < 0 && errno == EINTR) {
            goto retry;
        }

        // Parse the result
        if (written == 0) {
            errno = EOF;
            return "failed to read some data due to EOF";
        }
        if (written < 0) {
            return "failed to read some data";
        }

        // Update the buffer
        *pos += written;
        return NULL;
    }
}


/**
 * @brief Waits until the data has been flushed to the serial device
 * 
 * @param fd The file descriptor to flush
 * @return `NULL` on success or a static error string in case of a failure
 */
const char* serial_flush(int64_t fd) {
    int result = tcdrain((int)fd);
    if (result < 0) {
        return "failed to flush serial device";
    }
    return NULL;
}


/**
 * @brief Closes `fd`
 * 
 * @param fd The file descriptor to close
 */
void serial_close(int64_t fd) {
    close((int)fd);
}
