namespace Test;

public class Class1
{
    public static int Square(int x)
    {
        int res = 1;
        while (x > 0)
        {
            res *= 2;
            x -= 1;
        }
        return res;
    }
}
