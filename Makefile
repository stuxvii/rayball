DYLINK = -lraylib -lcurl
STATIC = -static raylib.a curl.a

BASE = g++ main.cpp -o rayball -std=c++20
RELFLAGS = -O3 -march=native -flto -ffast-math -DNDEBUG -s

debug:
	${BASE} ${DYLINK}
main:
	${BASE} ${RELFLAGS} ${DYLINK}
mainembed:
	${BASE} ${RELFLAGS} ${DYLINK} -DUSE_EMBEDDED_IMAGES
