#ifndef MPU9_H
#define MPU9_H

#define MPU9250_ADDRESS  0x69
#define DEVICE_ID  0x71
#define WHO_AM_I  0x75
#define PWR_MGMT_1  0x6B
#define INT_PIN_CFG  0x37
#define INT_ENABLE  0x38
// --- Accel ------------------
#define ACCEL_DATA  0x3B
#define ACCEL_CONFIG  0x1C
#define ACCEL_CONFIG2  0x1D
#define ACCEL_2G  0x00
#define ACCEL_4G  0x01 << 3
#define ACCEL_8G  0x02 << 3
#define ACCEL_16G  0x03 << 3
// --- Temp --------------------
#define TEMP_DATA  0x41
// --- Gyro --------------------
#define GYRO_DATA  0x43
#define GYRO_CONFIG  0x1B
#define GYRO_250DPS  0x00
#define GYRO_500DPS  0x01 << 3
#define GYRO_1000DPS  0x02 << 3
#define GYRO_2000DPS  0x03 << 3

#include "I2C_Device.h"

class MPU9: public I2C_Device{
    public:
        MPU9(int):I2C_Device();
        virtual ~MPU9();
        double[] accel();
        double[] gyro();
        double temp();
    private:
        // TODO: add read_xyz, conv, read16
};
