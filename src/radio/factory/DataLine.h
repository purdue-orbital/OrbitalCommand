//
// Created by nicholasball on 1/26/23.
//

/*
 * DataLine is a header file to make it easy to transmit and receive to/from data streams
 */

#ifndef RADIO_DATALINE_H
#define RADIO_DATALINE_H


#include "../stream/Data_Stream.h"


int __Num_Data_Lines__ = 0;
class DataLine
{
private:
    // Rx and Tx data streams
    Data_Stream *RX;
    Data_Stream *TX;
    int stream_num;

public:
    DataLine(double frequency, double sample_rate, double bandwidth)
    {
        double analog_filter = bandwidth;
        double digital_filter = 0;

        // calculate digital filter (the lowest analog filter can go is 5 MHz, so we set a digital filter if we need lower)
        if(bandwidth < 5e6)
        {
            analog_filter = 5e6;
            digital_filter = bandwidth;
        }

        // Make tx and rx streams
        RX = new Data_Stream(__Num_Data_Lines__, frequency, sample_rate,Radio_Method::RX,analog_filter,digital_filter);
        TX = new Data_Stream(__Num_Data_Lines__, frequency, sample_rate,Radio_Method::TX,analog_filter,digital_filter);

        stream_num = __Num_Data_Lines__;
        __Num_Data_Lines__++;

        Status("Creating data stream "+std::to_string(stream_num));
    }

    ~DataLine()
    {

    }


    /*!
     * Transmit data
     *
     * @param data vector of normalized I[0] and Q[1] values in that order (EX: [0] = I, [1] = Q, [2] = I, [3] = Q)
     */

    void Transmit(std::vector<float> data)
    {
        Status("Preparing to transmit data on stream "+std::to_string(stream_num));

        try
        {
            bool no_err = TX->Transmit(&data);
            if(!no_err) throw Error("Error while transmitting data on stream "+std::to_string(stream_num));

        } catch (...)
        {
            Status("Runtime error");
            throw Error("Error transmitting data from stream "+std::to_string(stream_num));
        }


    }

    /*!
     * Get data received from transmission
     *
     * @return vector of normalized I[0] and Q[1] values in that order (EX: [0] = I, [1] = Q, [2] = I, [3] = Q)
     */
    std::vector<float> Fetch()
    {
        Status("Fetching data on stream "+std::to_string(stream_num));

        try
        {
            return RX->Collect();

        } catch (...)
        {
            Status("Runtime error");
            throw Error("Error collecting data from stream "+std::to_string(stream_num));
        }


    }
};

#endif //RADIO_DATALINE_H
