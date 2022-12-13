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
    if (cfsetspeed(&tty, bauds) != 0) {
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
    return dup((int)fd);
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
 * @return `0` or `-1` on error 
 */
int32_t serial_read_buf(uint8_t* buf, size_t* pos, size_t capacity, int64_t fd) {
    // Return if the buffer is exhausted
    const size_t available = capacity - *pos;
    if (available == 0) {
        return 0;
    }

    // Read some data
    ssize_t read_ = read((int)fd, buf + *pos, available);
    if (read_ == 0) {
        errno = EOF;
        return -1;
    }
    if (read_ < 0) {
        return -1;
    }

    // Update the buffer
    *pos += read_;
    return 0;
}

/**
 * @brief Writes some data from `buf` and updates `pos` accordingly
 * 
 * @param fd The file descriptor to write to
 * @param buf The buffer to write to
 * @param pos The position within the buffer
 * @param capacity The total capacity of the buffer
 * @return `0` or `-1` on error  
 */
int32_t serial_write_buf(int64_t fd, const uint8_t* buf, size_t* pos, size_t capacity) {
    // Return if the buffer is exhausted
    const size_t available = capacity - *pos;
    if (available == 0) {
        return 0;
    }

    // Write some data
    ssize_t written = write((int)fd, buf + *pos, available);
    if (written == 0) {
        errno = EOF;
        return -1;
    }
    if (written < 0) {
        return -1;
    }

    // Update the buffer
    *pos += written;
    return 0;
}


/**
 * @brief Waits until the data has been flushed to the serial device
 * 
 * @param fd The file descriptor to flush
 * @return `0` or `-1` on error   
 */
int32_t serial_flush(int64_t fd) {
    int result = tcdrain((int)fd);
    if (result < 0) {
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
    close((int)fd);
}
