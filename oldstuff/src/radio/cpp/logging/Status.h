//
// Created by nicholasball on 1/24/23.
//

#ifndef RADIO_STATUS_H
#define RADIO_STATUS_H


#include "Log.h"

class Status : public Log
{
public:

    /*!
     * Create and log a status event
     *
     * @abstract Create and log a status event
     *
     * @param   what  The status event to record
    */

    Status(std::string what) : Log(what,"logs/Radio_Status.log") {};

};

#endif //RADIO_STATUS_H
