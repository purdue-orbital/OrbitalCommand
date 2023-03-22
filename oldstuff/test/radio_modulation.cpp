#include "../src/radio/factory/Modulator.h"
#include <iostream>


int main()
{
    Modulator mod;

    if(mod.Demod(mod.Mod("101",2e6,500e3,32e3),2e6,500e3,32e3) == "101")
    {
        std::cout<<"Modulation: PASSED"<<std::endl;
    }else{
        std::cout<<"Modulation: FAILED"<<std::endl;
    }

    return 0;
}