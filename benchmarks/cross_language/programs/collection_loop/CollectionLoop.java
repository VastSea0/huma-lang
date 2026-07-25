import java.util.ArrayList;

public final class CollectionLoop {
    public static void main(String[] args) {
        ArrayList<Long> values = new ArrayList<>(50_000);
        for (long i = 0; i < 50_000; i++) {
            values.add(i % 997L);
        }
        long total = 0;
        for (long value : values) {
            total += value;
        }
        System.out.println(total);
    }
}
