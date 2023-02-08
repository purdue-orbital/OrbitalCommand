//
// Created by nicholasball on 1/28/23.
//

#ifndef RADIO_IQ_H
#define RADIO_IQ_H
#include <vector>
#include <math.h>

class IQ
{
public:

    float I,Q;

    IQ(): I(0),Q(0) {};

    IQ(float i, float q) : I(i),Q(q) {}

    float amplitude()
    {
        return sqrt((I * I) + (Q * Q));
    }

    // make array of IQ data from float vector
    static std::vector<IQ> from_array(std::vector<float> arr)
    {
        std::vector<IQ> toReturn;
        toReturn.resize((int)(arr.size() / 2));

        // Make IQ data
        for(int i = 0; i < (arr.size() / 2); i++) toReturn[i] = IQ(arr[2 * i],arr[2 * i + 1]);

        return toReturn;
    }
};
#endif //RADIO_IQ_H
