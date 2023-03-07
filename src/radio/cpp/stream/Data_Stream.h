//
// Created by nicholasball on 1/24/23.
//


#ifndef RADIO_DATA_STREAM_H
#define RADIO_DATA_STREAM_H

#include <vector>
#include <thread>
#include <stdlib.h>
#include "Device.h"

enum Radio_Method {TX,RX};

long long GetTime()
{
    const auto p1 = std::chrono::system_clock::now();

    return  std::chrono::duration_cast<std::chrono::seconds>(p1.time_since_epoch()).count();
}

class Data_Stream
{
public:
    std::vector<float> Buffer;      // Buffer of values
protected:
    double Center_Frequency;              // The frequency in which the radio operates on
    double Sample_Rate;                   // Sample rate the radio works on
    int Channel;                          // The channel in which the data stream communicates on
    lms_stream_t Stream;                  // Data stream object
    Radio_Method Method;                  // Is this stream a receiving stream or a transmission stream

    /*!
     * TX loop of the radio
     *
     * @param stream A pointer to the data stream to use for buffer
     *
    */

    static void _RX_Loop(std::vector<float> *Buffer,lms_stream_t *Stream)
    {
        // Rx loop
        while (true) {
            // make array
            std::vector<float> buffer;
            buffer.resize(2e6 * 4 * 2);

            // read from radio (We can pretend the vector is an array)
            LMS_RecvStream(Stream, &buffer[0], (int)(buffer.size() / 2), NULL, 10000000);

            Buffer->insert( Buffer->end(), buffer.begin(), buffer.end());

            std::this_thread::sleep_for(std::chrono::seconds(1));
        }

    }
    
public:

    /*!
     * radio stream abstraction either to or from the radio
     *
     * @note How SDRs work are a little weird. For LimeSDR specifically, it has two channels, A and B but the code calls
     * them 0 and 1 respectfully. Each channel are full-duplex and can take one rx stream and one tx stream. So ensure
     * there is only one RX and one TX per a channel. A good resource to help understand more about SDRs in general is
     * this video https://youtu.be/xQVm-YTKR9s?t=238 by Andreas Spiess. I highly recommend watching to the end to
     * understand completely!
     *
     * @param   channel               The channel to add this stream to.
     * @param   center_frequency      The center frequency to analyze (In our situation Local Oscillator (LO) Frequency == center frequency, but this isn't always the case for other radios!)
     * @param   sample_rate           The rate to take a sample from
     * @param   method                Set whether this is a RX or TX stream
     * @param   analog_lpf_bandwidth  Set the analog filter bandwidth. Set this as cloe to the transmitted frequency as possible but can only go as low as 5 MHz
     * @param   digital_lpf_bandwidth Set the digital filter bandwidth. This can go as low as you want just and works like the analog version. However, utilizes more CPU power the lower you go
    */
    Data_Stream(int channel, double center_frequency, double sample_rate, Radio_Method method, double analog_lpf_bandwidth, double digital_lpf_bandwidth) : Channel(channel), Center_Frequency(center_frequency), Sample_Rate(sample_rate), Method(method)
    {
        Status("Stream fetching connected radio...");

        if(__RADIO__.address == NULL)
        {
            Device::getInstance();
        }

        try
        {
            // ensure the radio is connected before opening a data stream
            while(!__CONNECTED__){}

            // Mark if this stream TXs or RXs
            bool type = (method == Radio_Method::TX);


            Status("Initializing data stream...");

            Stream.channel = Channel;                   //channel number
            Stream.fifoSize = 256 * 1024;              //fifo size in samples
            Stream.throughputVsLatency = 0.5;           //0 min latency, 1 max throughput
            Stream.dataFmt = lms_stream_t::LMS_FMT_F32; // Take data in as f32s
            Stream.isTx = type;                         // Set channel type


            Status("Setting stream data...");



            if(LMS_EnableChannel(__RADIO__.address,type,Channel,true) != 0) throw Error("Error enabling channel "+std::to_string(Channel)+". (Is the radio damaged?)");
            if(LMS_SetLOFrequency(__RADIO__.address,type,Channel,Center_Frequency) !=  0) throw Error("Error setting Local Oscillating (LO) Frequency "+std::to_string(Center_Frequency)+" HZ. (Is it within range?)");
            if(LMS_SetSampleRate(__RADIO__.address,Sample_Rate,0) != 0) throw Error("Error setting sample rate "+std::to_string(sample_rate)+" HZ. (Is the sample rate in range?)");
            if(LMS_SetGaindB(__RADIO__.address,type,Channel,60) != 0) throw Error("Error setting gain to 0.7 of max. (Is the radio broken?)");

            if(!type)
            {
                LMS_SetAntenna(__RADIO__.address,type,Channel,LMS_PATH_LNAL);
            }

            if(analog_lpf_bandwidth > 0) {
                if (LMS_SetLPFBW(__RADIO__.address, type, Channel, analog_lpf_bandwidth) != 0) throw Error("Error setting low pass filter bandwidth to" + std::to_string(analog_lpf_bandwidth) +" HZ.(Is the bandwidth out of range?)");
            }
            if(digital_lpf_bandwidth > 0) {
                LMS_SetGFIRLPF(__RADIO__.address, type, Channel, true, digital_lpf_bandwidth);
            }

            if(LMS_Calibrate(__RADIO__.address,type,Channel,analog_lpf_bandwidth,0) != 0) throw Error("Error calibrating stream. (Was the data set correctly?)");

            Status("Starting stream...");

            if(LMS_SetupStream(__RADIO__.address,&Stream) != 0) throw Error("Error setting up stream. (Was the data set correctly?)");
            if(LMS_StartStream(&Stream) != 0) throw Error("Error starting stream. (Was the data set correctly?)");

            Status("Calibrating stream...");

            // if RX stream, start listen thread
            if(!type){
                Buffer = {};
                std::thread t(_RX_Loop,&Buffer,&Stream);
                t.detach();
            }

            Status("Stream running!");
        }
        catch (...)
        {
            Status("A runtime error occurred!");
            throw Error("A runtime error occurred while setting up data stream");

        }
    }

    ~Data_Stream()
    {
        if(LMS_StopStream(&Stream) != 0) Error("Error stopping stream (Was the radio removed?)");
        if(LMS_DestroyStream(__RADIO__.address,&Stream) != 0) Error("Error destroying stream (Was the radio removed?)");
    }

    /*!
     * Send transmission
     *
     * @note Ensure this stream is set to TX or an error will return
     *
     * @param   buffer             The buffer of data point in which to transmit (Should be in {[0] = I, [1] = Q, [2] = I, [3] = Q} format)
     *
     * @return Bool value if the transmission failed (false) or went through (true)
    */
    bool Transmit(std::vector<float> *buffer)
    {
        // ensure in tx mode
        if(this->Method != Radio_Method::TX)
        {
            Error("Attempted to transmit through a receive stream. (only transmit on TX streams)");
            return false;
        }

        Status("Transmitting data...");


        // transmit
        if(LMS_SendStream(&this->Stream,&((*buffer)[0]),buffer->size() / 2, nullptr,100000) == -1) throw Error("Error trying to transmit data (Are there strange numbers?)");

        Status("Transmitted data");

        return true;
    }

    /*!
     * Read transmissions from radio
     *
     * @note Ensure this stream is set to RX or an error will return
     *
     * @return array of IQ values in {[0] = I, [1] = Q, [2] = I, [3] = Q} format
    */
    std::vector<float> Collect()
    {
        // ensure in tx mode
        if(this->Method != Radio_Method::RX)
        {
            Error("Attempted to receive through a transmission stream. (only receive on RX streams)");
            return {};
        }

        // array to return
        std::vector<float> data = {};

        Status("Retrieving data...");

        data.insert(data.end(),Buffer.begin(),Buffer.end());

        Buffer.clear();

        Status("Retrieved data");

        return data;
    }
};

#endif //RADIO_DATA_STREAM_H
