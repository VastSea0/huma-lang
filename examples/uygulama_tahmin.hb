yükle "rastgele.hb";
yükle "renkler.hb";

baslat fonksiyon olsun {
    renkli_yaz("===== SAYI TAHMİN OYUNU =====", SARI + KALIN)
    "1 ile 100 arasında bir sayı tuttum."'yi yazdır
    
    hedef = r_tamsayı(1, 100) olsun
    deneme = 0 olsun
    tahmin = 0 olsun
    
    tahmin != hedef olduğu sürece {
        deneme = deneme + 1 olsun
        tahmin = sayıya_çevir(oku(deneme + ". Tahmininiz: ")) olsun
        
        tahmin < hedef ise {
            renkli_yaz("Daha BÜYÜK bir sayı girin.", MAVI)
        } yoksa tahmin > hedef ise {
            renkli_yaz("Daha KÜÇÜK bir sayı girin.", TURKUAZ)
        }
    }
    
    başarı_yaz("TEBRİKLER! " + deneme + " denemede buldunuz.")
}

baslat()
