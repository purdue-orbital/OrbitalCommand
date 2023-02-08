//
// Created by nicholasball on 1/26/23.
//

#ifndef RADIO_MODULATOR_H
#define RADIO_MODULATOR_H

#include <iostream>
#include <string>
#include <vector>
#include <math.h>
#include "../qol/IQ.h"

class Modulator
{

public:

    Modulator()
    {

    }

    std::string Demod(std::vector<float> data, double radio_sample_rate, double tone_freq, double mod_sample_rate)
    {
        std::vector<std::vector<IQ>> denoised = {};
        std::vector<IQ> hold = {};
        int counter = 0;

        auto lazy = IQ::from_array(data);

        
        // remove noise
        for(int i = 0; i < lazy.size(); i++)
        {
            float amp = lazy[i].amplitude();

            if (amp >= 0.3)
            {
                hold.push_back(lazy[i]);
                counter = 0;

            }else{

                if(counter == 5)
                {
                    denoised.push_back(hold);
                    hold = {};
                }

                counter++;
            }
        }

        denoised.push_back(hold);
        const double f_ratio = (tone_freq / radio_sample_rate);
        const double pi = acos(-1);
        std::string out = "";

        int Margin = radio_sample_rate / mod_sample_rate;

        for(int x = 0; x < denoised.size(); x++)
        {
            for(int y = 0; y < denoised[x].size(); y += Margin)
            {
                double w = 2 * pi * y * f_ratio;

                if(fabs(sin(w) - denoised[x][y].Q) < 0.1) out += "1";
                else out += "0";

            }
        }

        return out;

    }

    std::vector<float> Mod(std::string bin, double radio_sample_rate, double tone_freq, double mod_sample_rate)
    {
        const double pi = acos(-1);

        int Margin = radio_sample_rate / mod_sample_rate;

        std::vector<float> tx_buffer;
        tx_buffer.resize((long long)(2 * bin.size() * Margin));

        const double f_ratio = (tone_freq / radio_sample_rate);

        for (int i = 0; i < bin.size(); i++) {

            double w = 2 * pi * f_ratio * i;

            IQ iq();
        }

        return tx_buffer;
    }
};

#endif //RADIO_MODULATOR_H
