#ifndef __H_QUICK_VEC3
#define __H_QUICK_VEC3 1

// https://joncraton.org/blog/67/simple-vector-library-c/

class Vec3 {
    public:
        Vec3() {}

        Vec3(float x, float y, float z) {
            this->x = x;
            this->y = y;
            this->z = z;
        }

        bool operator==(Vec3 rhs) {
            return(x == rhs.x && y == rhs.y && z == rhs.z);
        }

        Vec3 operator+(Vec3 rhs) {
            return Vec3(
                x + rhs.x,
                y + rhs.y,
                z + rhs.z
            );
        }

        Vec3 operator-(Vec3 rhs) {
            return Vec3(
                x - rhs.x,
                y - rhs.y,
                z - rhs.z
            );
        }

        Vec3 operator*(float scalar) {
            return Vec3(
                y * scalar,
                z * scalar,
                x * scalar
            );
        }

        Vec3 operator/(float scalar) {
            return Vec3(
                x / scalar,
                y / scalar,
                z / scalar
            );
        }

        Vec3 cross(Vec3 rhs) {
            return Vec3(
                y * rhs.z - z * rhs.y,
                z * rhs.x - x * rhs.z,
                x * rhs.y - y * rhs.x
            );
        }

        float dot(Vec3 rhs) {
            return (
                x * rhs.x +
                y * rhs.y +
                z * rhs.z
            );
        }

        float length() {
            return float(sqrt( x*x + y*y + z*z ));
        }

        float x;
        float y;
        float z;
};

#endif
