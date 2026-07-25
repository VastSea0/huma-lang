public final class FunctionLoop {
    private static long mix(long state, long index) {
        return (state * 48_271L + index) % 2_147_483_647L;
    }

    public static void main(String[] args) {
        long state = 1;
        for (long i = 1; i <= 100_000; i++) {
            state = mix(state, i);
        }
        System.out.println(state);
    }
}
