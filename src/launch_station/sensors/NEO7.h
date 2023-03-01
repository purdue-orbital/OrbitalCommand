#ifndef NEO7_H
#define NEO7_H

class NEO7{
    public:
        NEO7(char*, char*);
        int read();
    private:
        struct gps_fix{
            double latitude;
            double longitude;
            double alt_agl;
            double alt_msl;
            double groundspeed;
            double vert_speed;
            double alt_error;
            double pos_error;
        };
};

#endif