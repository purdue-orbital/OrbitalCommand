#define BUFFER_SIZE 0x01  //1 byte buffer


class I2C_Device {
	public:
		I2C_Device(int, int);
		virtual ~I2C_Device();
		uint8_t dataBuffer[BUFFER_SIZE];
		uint8_t read_byte(uint8_t);
		uint8_t write_byte(uint8_t, uint8_t);
		uint8_t* read_block(uint8_t, int);

	private:
		int _i2caddr;
		int _i2cbus;
		void openfd();
		char busfile[64];
		int fd;
};