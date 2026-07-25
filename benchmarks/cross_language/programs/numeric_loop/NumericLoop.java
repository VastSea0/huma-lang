public final class NumericLoop {
    public static void main(String[] args) {
        long state = 1;
        long total = 0;
        for (long i = 1; i <= 200_000; i++) {
            state = (state * 1_664_525L + 1_013_904_223L) % 4_294_967_296L;
            total = (total + state) % 9_007_199_254_740_881L;
        }
        System.out.println(total);
    }
}
