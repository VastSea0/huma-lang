public final class BranchLoop {
    public static void main(String[] args) {
        long state = 1;
        long total = 0;
        for (long i = 1; i <= 200_000; i++) {
            state = state * 2 + i;
            if (state >= 1_000_000_000L) {
                state -= 1_000_000_000L;
            }
            if (state >= 1_000_000_000L) {
                state -= 1_000_000_000L;
            }
            total += state;
        }
        System.out.println(total);
    }
}
