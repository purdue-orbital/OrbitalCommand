//
// This file will handle direct communication from the radio and the program
//
// Device is a singleton as there can only be one instance of it for the sake of program stability
//

#include <future>
#include <thread>
#include <iostream>
#include <lime/LimeSuite.h>
#include "../logging/Error.h"
#include "../logging/Status.h"

#ifndef RADIO_DEVICE_H
#define RADIO_DEVICE_H

struct Radio
{
    lms_device_t* address = NULL;
};


bool __CONNECTED__ = false;

Radio __RADIO__;

class Device
{
public:

    static Device& getInstance()
    {
        static Device    instance;
        return instance;
    }
private:

    /*!
     * Check if we are connected to a radio
     *
     * @function isConnected
     *
     * @abstract Check if we are connected to a radio
     *
     * @result Boolean of whether or not we are connected to a radio (true = Connected and false = Not Connected)
    */
    bool isConnected()
    {
        return __CONNECTED__;
    }

    Device()
    {
        try {
            lms_info_str_t list[8];

            Status("Connecting to a radio...");

            // find radios
            if (LMS_GetDeviceList(list) == -1) {
                Status("Was unable to find any radios");
                throw Error("Unable to find radio(s) (Are the radio drivers installed?)");
            }

            // go through list of radios and try to connect to one
            for (int i = 0; i < 8; i++) {
                // try to connect to radio
                if (LMS_Open(&__RADIO__.address, list[i], NULL) != 0) {
                    // if not connected, update status
                    Status("Unable to connect to radio " + std::to_string(i + 1) + "...");

                } else {
                    Status("Found and connected to radio "+ std::string(list[i]));

                    // end loop once connected
                    break;
                }

                if (i == 7) {
                    Status("Unable to connect to a radio");
                    throw Error("Could not connect to a radio (Are the radio(s) plugged in?)");
                }
            }

            Status("Initializing radio...");



            // Initialize radio
            if (LMS_Init(__RADIO__.address) != 0) throw Error("Error initializing radio (Is the radio damaged?)");

            // mark as connected
            __CONNECTED__ = true;


        }catch(...)
        {
            __CONNECTED__ = false;

            Status("Error occurred. Stopping program...");
            throw Error("A runtime error has occurred!");
        }

    }

    ~Device()
    {
        __CONNECTED__ = false;

        LMS_Close(__RADIO__.address);
    }

public:
    Device(Device const&)          = delete;
    void operator=(Device const&)  = delete;

};


#endif //RADIO_DEVICE_H
