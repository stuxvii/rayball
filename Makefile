BASE = g++ main.cpp -o rayball -lraylib -lcurl -std=c++20
RELFLAGS = -O3 -march=native -flto -ffast-math -DNDEBUG -s

debug:
	${BASE}
main:
	${BASE} ${RELFLAGS}
mainembed:
	${BASE} ${RELFLAGS} -DUSE_EMBEDDED_IMAGES