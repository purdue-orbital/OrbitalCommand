//
// Created by nicholasball on 1/24/23.
//


#include "Log.h"

#ifndef RADIO_ERROR_H
#define RADIO_ERROR_H


class Error : public Log
{
public:

    /*!
     * Create and log an error
     *
     * @abstract Create and log an error
     *
     * @param   what  The error message to record
    */

    Error(std::string what) : Log(what,"logs/Radio_Errors.log") {};

};

#endif //RADIO_ERROR_H
